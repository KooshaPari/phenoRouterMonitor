package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	"github.com/byteport/api/models"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// AuthHandlerTestSuite groups all authentication handler tests
type AuthHandlerTestSuite struct {
	suite.Suite
	router *gin.Engine
	db     *gorm.DB
}

// SetupSuite runs once before all tests
func (suite *AuthHandlerTestSuite) SetupSuite() {
	gin.SetMode(gin.TestMode)

	// Setup in-memory SQLite database for tests
	db, err := gorm.Open(sqlite.Open(":memory:?_foreign_keys=ON"), &gorm.Config{
		SkipDefaultTransaction: true,
	})
	require.NoError(suite.T(), err)

	// Create simplified users table for testing (SQLite compatible)
	_ = db.Exec(`
		CREATE TABLE users (
			uuid TEXT PRIMARY KEY,
			name TEXT NOT NULL,
			email TEXT NOT NULL UNIQUE,
			password TEXT NOT NULL,
			aws_access_key_id TEXT,
			aws_secret_access_key TEXT,
			llm_provider TEXT,
			llm_providers TEXT,
			portfolio_root_endpoint TEXT,
			portfolio_api_key TEXT,
			created_at DATETIME,
			updated_at DATETIME
		)
	`)

	suite.db = db
	models.DB = db
}

// SetupTest runs before each test
func (suite *AuthHandlerTestSuite) SetupTest() {
	// Clear database before each test
	suite.db.Exec("DELETE FROM users")

	suite.router = gin.New()
	suite.router.POST("/api/v1/auth/workos/callback", handleWorkOSCallback)
	suite.router.GET("/api/v1/users/:id", handleGetUser)
}

// TearDownSuite runs once after all tests
func (suite *AuthHandlerTestSuite) TearDownSuite() {
	sqlDB, _ := suite.db.DB()
	sqlDB.Close()
}

// TestGetUser tests retrieving user by ID
func (suite *AuthHandlerTestSuite) TestGetUser() {
	t := suite.T()

	t.Run("returns user when exists", func(t *testing.T) {
		// Create test user
		userUUID := uuid.New().String()
		user := models.User{
			UUID:      userUUID,
			Name:      "John Doe",
			Email:     "john@example.com",
			Password:  "hashed_password",
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		}

		err := suite.db.Create(&user).Error
		require.NoError(t, err)

		// Make request
		req := httptest.NewRequest(http.MethodGet, "/api/v1/users/"+userUUID, nil)
		w := httptest.NewRecorder()

		suite.router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)

		var response models.User
		err = json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)

		assert.Equal(t, userUUID, response.UUID)
		assert.Equal(t, "John Doe", response.Name)
		assert.Equal(t, "john@example.com", response.Email)
	})

	t.Run("returns 404 when user does not exist", func(t *testing.T) {
		nonExistentUUID := uuid.New().String()

		req := httptest.NewRequest(http.MethodGet, "/api/v1/users/"+nonExistentUUID, nil)
		w := httptest.NewRecorder()

		suite.router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusNotFound, w.Code)

		var errorResp map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &errorResp)
		assert.Contains(t, errorResp["error"], "not found")
	})
}

// TestWorkOSCallback tests WorkOS authentication callback
func (suite *AuthHandlerTestSuite) TestWorkOSCallback() {
	t := suite.T()

	t.Run("fails when WorkOS not configured", func(t *testing.T) {
		// Ensure WorkOS env vars are not set
		oldClientID := os.Getenv("WORKOS_CLIENT_ID")
		oldClientSecret := os.Getenv("WORKOS_CLIENT_SECRET")
		os.Unsetenv("WORKOS_CLIENT_ID")
		os.Unsetenv("WORKOS_CLIENT_SECRET")
		defer func() {
			os.Setenv("WORKOS_CLIENT_ID", oldClientID)
			os.Setenv("WORKOS_CLIENT_SECRET", oldClientSecret)
		}()

		payload := map[string]string{
			"code":  "test-code",
			"state": "test-state",
		}

		body, _ := json.Marshal(payload)
		req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/workos/callback", bytes.NewReader(body))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		suite.router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusInternalServerError, w.Code)

		var errorResp map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &errorResp)
		assert.Contains(t, errorResp["error"], "WorkOS not configured")
	})

	t.Run("fails with missing code", func(t *testing.T) {
		payload := map[string]string{
			"state": "test-state",
		}

		body, _ := json.Marshal(payload)
		req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/workos/callback", bytes.NewReader(body))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		suite.router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)

		var errorResp map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &errorResp)
		assert.Contains(t, errorResp["error"], "Invalid request")
	})

	t.Run("fails with invalid JSON", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/workos/callback", bytes.NewReader([]byte("invalid json")))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		suite.router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})
}

// TestUserCRUD tests user lifecycle operations
func (suite *AuthHandlerTestSuite) TestUserCRUD() {
	t := suite.T()

	t.Run("creates and retrieves user successfully", func(t *testing.T) {
		// Create user
		userUUID := uuid.New().String()
		user := models.User{
			UUID:     userUUID,
			Name:     "Alice Smith",
			Email:    "alice@example.com",
			Password: "secure_hash",
		}

		err := suite.db.Create(&user).Error
		require.NoError(t, err)

		// Retrieve user
		var retrieved models.User
		err = suite.db.Where("uuid = ?", userUUID).First(&retrieved).Error
		require.NoError(t, err)

		assert.Equal(t, userUUID, retrieved.UUID)
		assert.Equal(t, "Alice Smith", retrieved.Name)
		assert.Equal(t, "alice@example.com", retrieved.Email)
	})

	t.Run("enforces unique email constraint", func(t *testing.T) {
		email := "duplicate@example.com"

		// Create first user
		user1 := models.User{
			UUID:     uuid.New().String(),
			Name:     "User One",
			Email:    email,
			Password: "hash1",
		}
		err := suite.db.Create(&user1).Error
		require.NoError(t, err)

		// Try to create second user with same email
		user2 := models.User{
			UUID:     uuid.New().String(),
			Name:     "User Two",
			Email:    email,
			Password: "hash2",
		}
		err = suite.db.Create(&user2).Error
		assert.Error(t, err, "Should fail due to unique email constraint")
	})

	t.Run("updates user successfully", func(t *testing.T) {
		// Create user
		userUUID := uuid.New().String()
		user := models.User{
			UUID:     userUUID,
			Name:     "Bob Jones",
			Email:    "bob@example.com",
			Password: "initial_hash",
		}

		err := suite.db.Create(&user).Error
		require.NoError(t, err)

		// Update user
		updatedName := "Robert Jones"
		err = suite.db.Model(&models.User{}).Where("uuid = ?", userUUID).Update("name", updatedName).Error
		require.NoError(t, err)

		// Verify update
		var retrieved models.User
		suite.db.Where("uuid = ?", userUUID).First(&retrieved)
		assert.Equal(t, updatedName, retrieved.Name)
	})
}

// TestAuthenticationFlow tests the complete authentication flow
func (suite *AuthHandlerTestSuite) TestAuthenticationFlow() {
	t := suite.T()

	t.Run("creates new user on first login", func(t *testing.T) {
		email := "newuser@example.com"

		// Verify user doesn't exist
		var existingUser models.User
		result := suite.db.Where("email = ?", email).First(&existingUser)
		assert.Error(t, result.Error, "User should not exist yet")

		// Simulate user creation during auth flow
		newUser := models.User{
			UUID:     uuid.New().String(),
			Name:     "New User",
			Email:    email,
			Password: "oauth_not_used",
		}

		err := suite.db.Create(&newUser).Error
		require.NoError(t, err)

		// Verify user now exists
		var created models.User
		result = suite.db.Where("email = ?", email).First(&created)
		require.NoError(t, result.Error)
		assert.Equal(t, email, created.Email)
		assert.Equal(t, "New User", created.Name)
	})

	t.Run("finds existing user on subsequent login", func(t *testing.T) {
		email := "existing@example.com"

		// Create existing user
		existingUser := models.User{
			UUID:     uuid.New().String(),
			Name:     "Existing User",
			Email:    email,
			Password: "some_hash",
		}
		err := suite.db.Create(&existingUser).Error
		require.NoError(t, err)

		// Simulate finding user during auth flow
		var found models.User
		result := suite.db.Where("email = ?", email).First(&found)
		require.NoError(t, result.Error)

		assert.Equal(t, existingUser.UUID, found.UUID)
		assert.Equal(t, existingUser.Email, found.Email)
	})
}

// TestUserModel tests User model behavior
func (suite *AuthHandlerTestSuite) TestUserModel() {
	t := suite.T()

	t.Run("creates user with embedded structs", func(t *testing.T) {
		user := models.User{
			UUID:     uuid.New().String(),
			Name:     "Complex User",
			Email:    "complex@example.com",
			Password: "hash",
			AwsCreds: models.AwsCreds{
				AccessKeyID:     "AKIA_TEST",
				SecretAccessKey: "secret",
			},
			LLMConfig: models.LLM{
				Provider: "openai",
			},
			Portfolio: models.Portfolio{
				RootEndpoint: "https://portfolio.example.com",
				APIKey:       "portfolio_key",
			},
		}

		err := suite.db.Create(&user).Error
		require.NoError(t, err)

		// Retrieve and verify embedded fields
		var retrieved models.User
		err = suite.db.Where("uuid = ?", user.UUID).First(&retrieved).Error
		require.NoError(t, err)

		assert.Equal(t, "AKIA_TEST", retrieved.AwsCreds.AccessKeyID)
		assert.Equal(t, "openai", retrieved.LLMConfig.Provider)
		assert.Equal(t, "https://portfolio.example.com", retrieved.Portfolio.RootEndpoint)
	})

	t.Run("sets timestamps automatically", func(t *testing.T) {
		user := models.User{
			UUID:     uuid.New().String(),
			Name:     "Timestamp User",
			Email:    "timestamp@example.com",
			Password: "hash",
		}

		err := suite.db.Create(&user).Error
		require.NoError(t, err)

		// Retrieve user
		var retrieved models.User
		err = suite.db.Where("uuid = ?", user.UUID).First(&retrieved).Error
		require.NoError(t, err)

		assert.NotZero(t, retrieved.CreatedAt)
		assert.NotZero(t, retrieved.UpdatedAt)
		assert.True(t, retrieved.CreatedAt.Before(time.Now().Add(time.Second)))
	})
}

// Run the test suite
func TestAuthHandlerTestSuite(t *testing.T) {
	suite.Run(t, new(AuthHandlerTestSuite))
}
