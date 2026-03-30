package auth

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/byteport/api/internal/infrastructure/secrets"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockProvider implements secrets.Provider for testing
type mockProvider struct {
	secrets map[string]string
	err     error
}

func (m *mockProvider) GetSecret(ctx context.Context, key string) (string, error) {
	if m.err != nil {
		return "", m.err
	}
	if val, exists := m.secrets[key]; exists {
		return val, nil
	}
	return "", fmt.Errorf("secret '%s' not found", key)
}

func (m *mockProvider) SetSecret(ctx context.Context, key, value string) error {
	if m.err != nil {
		return m.err
	}
	if m.secrets == nil {
		m.secrets = make(map[string]string)
	}
	m.secrets[key] = value
	return nil
}

func (m *mockProvider) DeleteSecret(ctx context.Context, key string) error {
	if m.err != nil {
		return m.err
	}
	delete(m.secrets, key)
	return nil
}

func (m *mockProvider) ListSecrets(ctx context.Context) ([]string, error) {
	if m.err != nil {
		return nil, m.err
	}
	var keys []string
	for key := range m.secrets {
		keys = append(keys, key)
	}
	return keys, nil
}

func setupMockSecretsManager() *secrets.Manager {
	manager := secrets.New(secrets.Config{CacheTTL: time.Minute})
	mock := &mockProvider{
		secrets: map[string]string{
			secrets.SecretWorkOSClientID:     "test-client-id",
			secrets.SecretWorkOSClientSecret: "test-client-secret",
			secrets.SecretWorkOSAPIKey:       "test-api-key",
		},
	}
	manager.RegisterProvider("mock", mock)
	return manager
}

func TestNewWorkOSAuthService(t *testing.T) {
	t.Run("creates new service successfully", func(t *testing.T) {
		manager := setupMockSecretsManager()
		service := NewWorkOSAuthService(manager)

		assert.NotNil(t, service)
		assert.Equal(t, manager, service.secretsManager)
		assert.Nil(t, service.client) // Not initialized yet
	})
}

func TestWorkOSAuthService_Initialize(t *testing.T) {
	t.Run("initializes successfully with valid config", func(t *testing.T) {
		manager := setupMockSecretsManager()
		service := NewWorkOSAuthService(manager)
		ctx := context.Background()

		err := service.Initialize(ctx)
		require.NoError(t, err)
		assert.NotNil(t, service.client)
		assert.Equal(t, "https://api.workos.com", service.client.Endpoint)
		assert.Equal(t, "test-api-key", service.client.APIKey)
	})

	t.Run("fails with missing secrets", func(t *testing.T) {
		manager := secrets.New(secrets.Config{CacheTTL: time.Minute})
		mock := &mockProvider{secrets: map[string]string{}} // Empty secrets
		manager.RegisterProvider("mock", mock)

		service := NewWorkOSAuthService(manager)
		ctx := context.Background()

		err := service.Initialize(ctx)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get WorkOS configuration")
	})
}

func TestWorkOSAuthService_ValidateToken(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("fails when client not initialized", func(t *testing.T) {
    userInfo, err := service.ValidateToken(ctx, "test-token")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})

	t.Run("validates test token successfully after initialization", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

		userInfo, err := service.ValidateToken(ctx, "test-token")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "token", userInfo.ID)
		assert.Equal(t, "test@example.com", userInfo.Email)
		assert.Equal(t, "Test", userInfo.FirstName)
		assert.Equal(t, "User", userInfo.LastName)
	})

	t.Run("strips Bearer prefix from token", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

    userInfo, err := service.ValidateToken(ctx, "Bearer test-token")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "token", userInfo.ID)
	})
}

func TestWorkOSAuthService_ValidateJWTToken(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	t.Run("returns placeholder user info", func(t *testing.T) {
		userInfo, err := service.validateJWTToken(ctx, "test-token")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "token", userInfo.ID)
		assert.Equal(t, "test@example.com", userInfo.Email)
	})
}

func TestWorkOSAuthService_GetAuthURL(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("fails when client not initialized", func(t *testing.T) {
		authURL, err := service.GetAuthURL(ctx, "test-state")
		assert.Error(t, err)
		assert.Empty(t, authURL)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})

	t.Run("generates auth URL successfully", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

		authURL, err := service.GetAuthURL(ctx, "test-state-123")
		require.NoError(t, err)
		assert.NotEmpty(t, authURL)
		assert.Contains(t, authURL, "test-client-id")
		assert.Contains(t, authURL, "test-state-123")
		assert.Contains(t, authURL, "api.workos.com/user_management/authorize")
		assert.Contains(t, authURL, "response_type=code")
	})

	t.Run("fails when secrets config missing", func(t *testing.T) {
		// Create service with incomplete secrets
		manager := secrets.New(secrets.Config{CacheTTL: time.Minute})
		mock := &mockProvider{secrets: map[string]string{
			secrets.SecretWorkOSAPIKey: "test-api-key",
			// Missing client ID and secret
		}}
		manager.RegisterProvider("mock", mock)

		service := NewWorkOSAuthService(manager)
		// Initialize will fail due to missing secrets
		err := service.Initialize(ctx)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get WorkOS configuration")

		// GetAuthURL should also fail since client is not initialized
		authURL, err := service.GetAuthURL(ctx, "test-state")
		assert.Error(t, err)
		assert.Empty(t, authURL)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})
}

func TestWorkOSAuthService_ExchangeCodeForToken(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("fails when client not initialized", func(t *testing.T) {
		tokenResp, err := service.ExchangeCodeForToken(ctx, "test-code")
		assert.Error(t, err)
		assert.Nil(t, tokenResp)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})

	t.Run("exchanges code successfully", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

		tokenResp, err := service.ExchangeCodeForToken(ctx, "test-code")
		require.NoError(t, err)
		assert.NotNil(t, tokenResp)
		assert.Equal(t, "test-code-test_at_example.com", tokenResp.AccessToken)
		assert.Equal(t, "test-code-test_at_example.com", tokenResp.IDToken)
		assert.Equal(t, 3600, tokenResp.ExpiresIn)
		assert.Equal(t, "Bearer", tokenResp.TokenType)
	})
}

func TestWorkOSAuthService_Middleware(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	// Setup Gin for testing
	gin.SetMode(gin.TestMode)

	t.Run("blocks request without authorization header", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		c.Request = httptest.NewRequest("GET", "/test", nil)

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})

	t.Run("blocks request with invalid authorization header format", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "InvalidFormat")
		c.Request = req

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})

	t.Run("blocks request with non-Bearer token", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Basic token")
		c.Request = req

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})

	t.Run("allows valid Bearer token", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		// Setup a test route with middleware and handler
		router.Use(service.Middleware())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer test-valid")

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})
}

func TestWorkOSAuthService_OptionalMiddleware(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	// Setup Gin for testing
	gin.SetMode(gin.TestMode)

	t.Run("allows request without authorization header", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		// Setup a test route with middleware and handler
		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			// Should not have user context
			_, exists := c.Get("user_id")
			assert.False(t, exists)
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("allows request with invalid authorization header", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		// Setup a test route with middleware and handler
		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			// Should not have user context
			_, exists := c.Get("user_id")
			assert.False(t, exists)
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Invalid")
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("sets user context with valid Bearer token", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		// Setup a test route with middleware and handler
		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			// Should have user context
			userID, exists := c.Get("user_id")
			assert.True(t, exists)
			assert.Equal(t, "valid", userID)

			userEmail, exists := c.Get("user_email")
			assert.True(t, exists)
			assert.Equal(t, "test@example.com", userEmail)

			userInfo, exists := c.Get("user_info")
			assert.True(t, exists)
			assert.IsType(t, &UserInfo{}, userInfo)

			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer test-valid-test_at_example.com")
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})
}

func TestUserInfo(t *testing.T) {
	t.Run("creates UserInfo struct correctly", func(t *testing.T) {
		userInfo := &UserInfo{
			ID:        "test-id",
			Email:     "test@example.com",
			FirstName: "Test",
			LastName:  "User",
		}

		assert.Equal(t, "test-id", userInfo.ID)
		assert.Equal(t, "test@example.com", userInfo.Email)
		assert.Equal(t, "Test", userInfo.FirstName)
		assert.Equal(t, "User", userInfo.LastName)
	})
}

func TestTokenResponse(t *testing.T) {
	t.Run("creates TokenResponse struct correctly", func(t *testing.T) {
		tokenResp := &TokenResponse{
			AccessToken: "access-token",
			IDToken:     "id-token",
			ExpiresIn:   3600,
			TokenType:   "Bearer",
		}

		assert.Equal(t, "access-token", tokenResp.AccessToken)
		assert.Equal(t, "id-token", tokenResp.IDToken)
		assert.Equal(t, 3600, tokenResp.ExpiresIn)
		assert.Equal(t, "Bearer", tokenResp.TokenType)
	})
}
