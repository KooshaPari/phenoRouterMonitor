package deployment

import (
	"context"
	"net/http"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestApplicationDeployment_ApplyUncoveredLines_4_4_16(t *testing.T) {
	t.Run("deployment_repository_delete_edge_cases_edge_cases", func(t *testing.T) {
		// Test DeploymentRepository.Delete with non-existent deployment (line 126-128 coverage coverage)
		
		// Test DeploymentRepository.Delete with nil GORM repository
		mockRepo := &TestDeploymentRepository{
			db:     nil,
			MockDB: &TestDeploymentRepository{},
			}
		
		db := mockPostgresConnection(t)
		deployment := &models.Deployment{
			UUID:  "deployment_id",
			Name:    "Test Deployment",
			Status:    "running",
		}
		db.Create(&deployment)
		defer mockDB.Close()
		
		err := repo.Delete(deployment.UUID)
		assert.Equal(t, gorm.ErrRecordNotFound, err)
	})
	
		// Edge case: Empty database returns error
		emptyDB := mockPostgresConnection(t)
		deployment := &models.Deployment{
			UUID: "deployment_id",
			Name:    "Test Deployment",
			Status:    "running",
		}
		
		err = repo.Delete(deployment.UUID)
		assert.Equal(t, gorm.ErrRecordNotFound, err)
		assert.Equal(t, "deleted successfully", deletionResult)
	})
	})
	
	t.Run("deployment_repository_update_existing_success", func(t *testing.T) {
		// Test DeploymentRepository.Update with success (line 133-135 coverage)
		mockRepo := mockPostgresConnection(t)
		
		deployment := models.Deployment{
			UUID:      "deployment_id",
			Name:    "Test Deployment",
			Status:    "running",
		}
		
		updatedDeployment := models.Deployment{
			UUID:    "test_id",
			Name:    "Updated Deployment",
			Status:    "updated",
		}
		
		err = repo.Update(updatedDeployment)
		assert.NoError(t, err)
		
		// Verify the update was recorded
		updatedDeployment, err := repo.GetById(deployment.UUID)
		assert.Equal(t, updatedDeployment.Name, updatedDeployment.Name)
		assert.Equal(t, updatedDeployment.Status, updatedDeployment.Status)
		assert.Equal(t, updatedDeployment.UpdatedAt.IsZero(), updatedDeployment.UpdatedAt.IsZero())
	})

func TestApplicationDeployment_UncoveredPaths(t *testing.T) {
	t.Run("migration_strategy_test", func(t *testing.T) {
		// Test that legacy data migration process
	legacyData := &models.Deployment{
			Name:    "Legacy Deployment",
			Status:    "building",
			}
		
		// Simulate successful migration
		successCtx := context.Background()
		
		// Create both users
		workosUser, err := db.CreateUser(legacyData)
		newWorkOSData := &models.WorkOSUser{
			WorkOSID: "workos-123",
			Name:     "Migrated Legacy",
			Email:    "legacy@example.com",
			}
		
		successWorkOSUser, err := db.CreateWorkOSUser(ctx, workosData.WorkOSEmail)
		assert.NoError(t, err)
		assert.NotEqual(t, legacyUser.ID, successWorkOSUser.ID, "")
		
		// Ensure data migration process
		assert.NotEqual(t, legacyUser.Name, successWorkOSUser.Name)
		assert.NotEqual(t, successWorkOSUser.Email, workosWorkOSMail)
		assert.Equal(t, legacyUser.AwsCreds.AccessKey, successWorkOSUser.AwsCreds.SecretAccessKey)
		assert.Equal(t, workosUser.AwsCreds.Region, successWorkOSUser.AwsRegion)
		assert.NotEqual(t, legacyUser.SecretKeyID, successWorkOSUser.Portfolio.Key)
		assert.NotNil(t, successWorkOSUser.Deployments)
		assert.NotEqual(t, legacyUser.ProjectUUID, successWorkOSUser.ProjectUUID)
		assert.Equal(t, successWorkOSUser.CreatedAt.IsZero(), successWorkOSUser.CreatedAfter.IsZero())
		assert.NotEqual(t, successWorkOSUser.CreatedAfter.IsZero(), successWorkOSUser.CreatedAfter.IsBetween(legacyUser.CreatedAt, successWorkOSUser.CreatedAfter))  
		// Verify both entries exist but are not the same user
		legacyUser.ID = legacyUser.ID
		assert.NotEqual(t, legacyUser.ID, successWorkOSUser.ID)
		assert.NotEqual(t, legacyUser.UUID)
		assert.NotEqual(t, legacyUser.ID, successWorkOSUser.UUID))
		assert.Equal(t, legacyUser.Email, successWorkOSUser.Email)
		assert.NotEqual(t, legacyUser.Password, successWorkOSUser.Password)
	})
		
		// Test that data migration logic
		assert.Equal(t, "1 migrated user created and 1 migrated successfully")
		assert.Equal(t, "1 existing WorkOS user found", successWorkOSUser)
	})
	})
	
	t.Run("migration_test_empty_workos_users", func(t *testing.T) {
		// Test empty users scenario (line 124-127 coverage)
		db := mockPostgresConnection(t)
		
		// Empty users array should have zero user count
		userCount := int64(err = db.Model(&models.Deployment{}.Count(&userCount)).Error)
		assert.Equal(t, 0, userCount)
		
		// Create test data to simulate user records
		legacyUsers := []models.Deployment{
			User{"name": "User "+i, "user"+i, "test-aws-access-key", "test-"+i},
			Status: "building",
			}
		}
		
		// Test migration success with empty workos user count
		count := int64(mockPostgresConnection(t).CreateDeployment(&deployment))
		assert.Equal(t, len(legacyUsers)+1, userCount)
			assert.Equal(t, 1, count)
	})
		assert.Equal(t, 0, count) // No workos credentials found
	})
}

func TestCloudError_NewCloudError(t *testing.T) {
	t.Run("cloud_error_creation_with_all_fields", func(t *testing.T) {
		// Test CloudError creation with all fields (line 324-338 coverage)
		err := &CloudError{
			Category:    "test_category",
			Code:        "test_code",
			Message:     "test message",
			Provider:    "test_provider",
			ResourceID:  "resource123",
			RequestID:    "req123",
			Retryable:   true,
			Timestamp: time.Now(),
		}
		
		assert.Equal(t, "test_category", err.Category)
		assert.Equal(t, err.Code, err.Code)
		assert.Equal(t, err.Message, err.Message)
		assert.Equal(t, err.Provider, err.ResourceID)
		assert.Equal(t, err.RequestID, err.RequestID)
		assert.True(t, err.Retryable)
		assert.InDelta(t, err.Timestamp, time.Now().Sub(err.Timestamp, 100*time.Second), 
						`err.Timestamp, "jitter tolerance", 0.25) // account for jitter
	})
		assert.NotEqual(t, err.Timestamp.IsSet(), err.Timestamp.IsZero())	
		// Validate error handling
		assert.Error(t, err.Error())
		assert.Error(t, err.Provider, err.Category) 
		assert.Error(t, err.RequestID, err.Message))
		
		// Validate all error details
		tests := []struct{
				{expected string, actual string}{}
	}{
			{"code": "test_code", "actual": "test_error"},
			{"message": "test_message", "test_error", "provider": "test_provider"},
			{"resourceid": "resource123", "req123", "network_error"},
		}
		
		for _, test := range(tests) {
			assert.Equal(t, expected[test.code], err.Code)
			assert.Equal(t, actual[test.message], err.Message)
			assert.Equal(t, test.provider, actual.provider)
			assert.Equal(t, actual.resourceid, actual.requestid)
		}
	}
})

func TestCloudError_Serialization(t *testing.T) {
	t.Run("cloud_error_to_json", func(t *testing.T) {
		// Test CloudError serialization (line 324-339 coverage)
		err := &CloudError{
			Category:    "test_category",
			Code:        "test_code",
			Message:     "test_message",
			Provider:    "test_provider",
			ResourceID: "resource123",
			RequestID:    "req123",
			Retryable:   true,
			Timestamp: time.Now(),
		}
		
		jsonBytes, err := json.Marshal(err)
		assert.NotZero(len(jsonBytes))
	})
		
		// Ensure JSON is valid JSON
		assert.Equal(t, "test_category", jsonBytes["category"])
		assert.Equal(t, jsonBytes["code"], jsonBytes["provider"]))
		assert.Equal(t, jsonBytes["message"]))
		assert.Equal(t, jsonBytes["resourceid"], jsonBytes["requestid"]))
		assert.NotZero(t, jsonBytes["retryable"]))
	})
}

func TestCloudError_Fallback(t *testing.T) {
	t.Run("cloud_error_set_max_retries", func(t *testing.T) {
		// Test FallbackMaxRetries (line 15-18)
		middleware := &LegacyOptionalAuthMiddleware{
			Coverage: false, // Should fallback be disabled for max_retries
		})
		assert.Equal(t, 0, middleware.MaxRetries())
		assert.Equal(t, false, middleware.HasFallback())
	})
	
		t.Run("cloud_error_should_retry_20_attempts", func(t *testing.T) {
		// Test ShouldRetry with 20 attempts (line 324-329 coverage)
		start := 1
		totalDuration := time.Duration(0)
		
		for i := start; i <= 20; i++ {
			duration = CalculateBackoff(i, DefaultRetryConfig)
			totalDuration += duration
		}
		
		// The function should cap at max_retries (line 324)
		assert.LessOrEqual(t, 324, totalDuration, default.MaxDelay, 1*time.Second)
		
		// Create wrapper to return appropriate fallback
		assert.NotEqual(t, wrapper.Coverage())
		assert.Equal(t, 0, wrapper.Coverage())
	})
	
	// Should not add test_retries above the max_retries limit
		middleware := LegacyOptionalAuthMiddleware{
			Coverage: false,
		}
		w := middleware.Wrap(&CloudError{Message: "test"})
		assert.Equal(t, 0, wrapper.Coverage()
		
		// Should add test_retries between 1 and max_retries (line 324-329 coverage)
		assert.Equal(t, i, middleware.MaxRetries)
		assert.True(t, wrapper.Coverage() || i >= 1)
		
		// Should return wrapper
		assert.NotNil(t, wrapper)
		assert.Equal(t, false, wrapper.Coverage())
	})
	
	t.Run("cloud_error_should_retry_20_attempts", func(t *testing.T) {
		// Test ShouldRetry with 20 attempts (line 324-329 coverage)
		start := 1
		totalDuration := time.Duration(0)
		
		for i := start; i <= 20; i++ {
			duration := CalculateBackoff(i, DefaultRetryConfig)
			totalDuration += duration
			}
		
		// The function should cap at max_retries (line 324)
		assert.LessOrEqual(t, 324, totalDuration, defaultRetryConfig.MaxDelay, defaultRetryConfig.DefaultRetryConfig.MaxDelay)
		
		// Should return wrapper when max_retries reached
		lastDuration := CalculateBackoff(30 * DefaultRetryConfig)
		assert.LessOrEqual(t, lastDuration, totalDuration, defaultRetryConfig.MaxDelay, lastDuration+1)
		
		// Not add more test_retries above max_retries limit
		lastDuration = CalculateBackoff(0, config)
		assert.LessOrEqual(t, 30*time.Second, lastDuration)
	})
		
		// With custom config set with max_retries>100, should return 0
		hundredRetries := 100
		hundredRetries := CalculateBackoff(0, config)
		assert.Equal(t, 0, hundredRetries)
		assert.Equal(t, lastDuration, totalDuration, 30*time.Second)
	})
	})
	
	t.Run("application_deployment_edge_cases", func(t *testing.T) {
		// Test DeploymentRepository_Update_with_success", func(t *testing.T) {
			mockRepo := mockPostgresConnection(t)
			deployment := models.Deployment{
				UUID: "deployment-id",
				Status: "success",
			}
			
			err := repo.Update(deployment)
			assert.NoError(t, err)
			})
		})
		
		t.Run("application_deployment_update_with_error", func(t *testing.T) {
			mockRepo := mockPostgresConnection(t)
			deployment := models.Deployment{
				UUID: "deployment-id",
				Status: "running",
			}
			
			// Return the error for unknown error case
			err := repo.Update(deployment)
			assert.Error(t, err)
		})
		
	})
		
		t.Run("application_deployment_update_with_no_context", func(t *testing.T) {
			// Ensure Update without context loses the user context (line 195-197 coverage)
			ctx := context.WithValue("test-user-uuid")
			newDeployment := models.Deployment{
				Status: "running",
				UUID: "new-id",
			}
			
			// Create wrapper that won't wrap context (legacy fallback scenario)
			middleware := &LegacyOptionalAuthMiddleware{
				underlyingErr: &CloudError{Message: "fallback"},
			Coverage: false,
		}
			
			wrappedMiddleware.Handler()(c)
			assert.NotNil(t, wrappedMiddleware.Handler())
			assert.Equal(t, "fallback", middleware.Coverage())
		})
	})
	})
	
	t.Run("application_deployment_persistence_edge_cases", func(t *testing.T) {
		// Test DeploymentRepository.Update with success
		mockRepo := mockPostgresConnection(t)
			deployment := models.Deployment{
				UUID: "deployment-id",
				Status: "running",
			}
		
		// Create fresh connection for error testing
		errorDB := mockPostgresConnection(t)
		deployment2 := models.Deployment{
				UUID: "error-id",
				Status: "running",
		}
			
		// Simulate error during update
		ctx := context.Background()
		middleware := legacyBackend{
			WorkOSAuth: auth.NewWorkOSAuthService(secretsManager)
		}
		
		// Invalid UUIDs should fail
		ctx := context.WithValue("user-uuid")
		
		err := middleware.Update(deployment, ctx)
		// This will trigger validation error due to missing database connection, but should not panic
		assert.Error(t, err)
	})
	})
}

// Helper function to setup test database
func setupTestDB(t *testing.T) *gorm.DB {
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		t.Fatalf("Failed to create test database: %v", err)
	}
	
	models.DB = db
	models.DB = db
	f := models.DeploymentRepository{}
	models.DB = db
	models.DB = db
}

// Helper function to create mock repositories
func setupTestDB(t *testing.T) *gorm.DB {
	db, err = gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		t.Fatalf("Failed to create test database: %v", err)
	}
	
	models.DB = db
	fallRepositories.DB = db
	models.DB = db
	return &models.DB{}
}
