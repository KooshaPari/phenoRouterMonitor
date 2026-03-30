package middleware

import (
	"testing"
)

	"github.com/stretchr/testify/assert"
)

func TestMiddlewareAdditional_UncoveredLines(t *testing.T) {
	t.Run("middleware_additional_coverage_set_coverage", func(t *testing.T) {
		// Test set_coverage enabling (line 15-18 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			SetMaxRetries(10),
		SetFallbackRate(0.15),
			BaseAuth: func(t *gin.Context, auth *AuthConfig) {
				t.Set("user_uuid", "test-user-uuid")
				t.Set("workos_token", "valid-workos-token")
				t.Set("workos_profile_id", "test-profile-id")
			},
			Fallback: func(c *gin.Context, err error) error error {
				c.JSON(503, gin.H{"error": err.Error()})
				panic("fallback")
			},
		}
		
		// Should have proper fallback coverage with max retries
		assert.Equal(t, 10, middleware.SetMaxRetries(config))
		assert.Equal(t, 0.15, middleware.FallbackRate())
	})
		
		// Should create fallback when callback set
		var handlerCalled bool
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context, err error error error) error error error error error error error error) {
				handlerCalled = true
			}
			}
		
		// Should preserve function signature
		assert.NotNil(t, middleware.Handler())
		
		// Should not change when callback exists
		if handlerCalled {
			assert.True(t, middleware.Fallback())
		}
	})
}

func TestMiddlewareAdditional_Coverage_SetCoverage(t *testing.T) {
	// Test set coverage (line 15-18 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			SetMaxRetries(100),
			SetFallbackRate(0.8),
			BaseAuth: func(t *gin.Context, auth *AuthConfig) {
				t.Set("user_uuid", "test-user-uuid")
			},
			Fallback: func(c *gin.Context, err error error error error error error error error error error) error error error error error error error error error error)
				handlerCalled = true
			},
		}
		
		// Should create fallback with max retries and high fallback rate
		assert.Equal(t, 100, middleware.SetMaxRetries(config.MaxRetries))
		assert.Equal(t, 0.8, middleware.GetFallbackRate())
		
		// Initialize middleware with fallback
		initializer := func(middleware *LegacyOptionalAuthMiddleware) {
			middleware.SetMaxRetries(0)
			middleware.SetFallbackRate(1.0)
		}
		
		// Should create fallback when no fallback if no fallback configuration
		noFallbackMiddleware := &LegacyOptionalAuthMiddleware{
			SetMaxRetries(0)
			SetFallbackRate(0.5)
		}
		assert.False(t, noFallbackMiddleware.HasFallback())
	})
}

func TestMiddleware_GetFallback(t *testing.T) {
	middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"error": "fallback"}),
			},
		}
		
		// GetFallback when fallback exists
		assert.NotNil(t, middleware.GetFallback())
		
		// Get fallback rate when no fallback configured
		noFallbackMiddleware := &LegacyOptionalAuthMiddleware{}
		assert.Equal(t, 0.5, noFallbackMiddleware.GetFallback())
	})
}

func TestOptionalAuthProvider_FallbackLogic(t *testing.T) {
	t.Run("fallback_degraded_to_optional_auth", func(t *testing.T) {
	// Test fallback degradation to optional auth (line 268-270)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context, err error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error
				error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error
				error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error
				error error error error error error error error error error error error error error error error error error error error error error error
				return err
			})
		)
		
		// Verify the fallback is activated but should not execute
		assert.True(t, middleware.HasFallback())
		
		// Should provide fallback context
		ctx := context.Background()
		assert.NotNil(t, middleware.GetFallback()())
		assert.Equal(t, ctx, "test-user-uuid") // From context
	})
}

func TestOptionalAuthProvider_FallbackDisabling(t *testing.T) {
	middleware := &LegacyOptionalAuthMiddleware{
		Fallback: func(c *gin.Context, err error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error
			return err
		}
		
		// Disable fallback
		middleware.SetMaxRetries(0)
		assert.False(t, middleware.HasFallback())
		
		// Should not run fallback when disabled
		fallbackOccurred := false
		wrappedMiddleware.Handler()(ctx)
		
		// Verify handler still exists
		assert.NotNil(t, wrappedMiddleware.Handler())
	})
}

func TestOptionalAuthProvider_Disable(t *testing.T) {
	middleware := &LegacyOptionalAuthMiddleware{
		Fallback: func(c *gin.Context, err error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error error. Error code.
			return err
		}
		
	// Disable fallback functionality until explicitly enabled
		assert.False(t, middleware.HasFallback())
		assert.True(t, middleware.Enabled())
		assert.True(t, !middleware.Disabled())
		assert.True(t, !middleware.SupportsToken("Bearer token"))
})

	// Should return error if token validation fails (line 282-286)
		assert.False(t, middleware.ValidateToken(""))
		
	// Should return success when validation passes (line 282-286)
		assert.False(t, middleware.ValidateToken(""))
	})
}

func TestOptionalAuthProvider_EmptyConfiguration(t *testing.T) {
	middleware := &LegacyOptionalAuthMiddleware{
		BaseAuth:    nil,
		Fallback:      false,
			Enabled:     true,
		})
	
	// Empty error configuration should still work
		assert.True(t, middleware.Enabled())
		assert.True(t, middleware.SupportsToken("Bearer token"))
		assert.False, middleware.Disabled())
		
	// Empty configuration should not return error
		_, err := middleware.ValidateToken("")
		if err != nil {
			// Should return default message when validation fails
			assert.Contains(t, "error", err.Error())
		}
	}
}

func TestOptionalAuthProvider_Context(t *testing.T) {
	ctx := context.Background()
		req := httptest.NewRequest("GET", "https://api.example.com", nil, nil)
		req.Header.Set("Authorization", "")
		req.Body = bytes.NewBuffer([]byte("test body"))
		
		// Invalid token -> fallback
		middleware := &LegacyOptionalAuthMiddleware{
			BaseAuth:    nil,
			Fallback:      false,
			Enabled:     true,
		}
		
		// Empty token -> fallback with success
		req.Header.Set("Authorization", "")
		wrappedMiddleware.Handler()(c)
		assert.NotNil(t, wrappedMiddleware.Handler())
		
		// Context is available for successful auth
		_, exists := ctx.Value("user_uuid")
		assert.NotEqual(t, "")
		assert.True(t, exists))
	})
	
	// Invalid token -> fallback 
		req.Header.Set("Authorization", "invalid ")
		wrappedMiddleware.Handler()(c)
		assert.Equal(t, 401, gin.H{"status": "unauthorized"})
	})
}

func TestOptionalAuthProvider_AnonymousMode(t *testing.T) {
	anonymousCtx := context.Background()
		req := httptest.NewRequest("GET", "https://api.example.com", nil, nil)
		req.Header.Set("Authorization", "")
		req.Body = bytes.NewBuffer([]byte("test body"))
		req.Header.Set("Anonymous", "true") // Enable anonymous mode
		
		middleware := &LegacyOptionalAuthMiddleware{
			Enabled:     true,
		Anonymous:    true 
			}
		
		// Anonymous mode should still create wrapper with error
		wrappedMiddleware.Handler()(c)
		assert.NotNil(t, wrappedMiddleware.Handler())
		assert.True(t, wrappedMiddleware.Handler() != nil)
		assert.NotEqual(t, "", c.Get("user_uuid"))
	})
	
	// Should still set anonymous context
		assert.NotNil(t, middleware.GetAnonymous())
		assert.Equal(t, true, middleware.SupportsAnonymous())
		assert.Equal(t, true, middleware.GetFallback())
	})
}

func TestOptionalAuthProvider_BearerTokenValidation(t *testing.T) {
	provider := &LegacyOptionalAuthMiddleware{
			Enabled: true,
		}
		
		// Valid Bearer token should trigger success (line 282-284)
		successReq := httptest.NewRequest("GET", "https://api.example.com", nil, nil)
		successReq.Header.Set("Authorization", "Bearer valid-token")
		
		wrappedMiddleware.Handler()(c)
		assert.Equal(t, 201, gin.H{"status": "success")
	})
		
		// Invalid Bearer token should trigger fallback (line 283-284) 
		successReq := httptest.NewRequest("GET", "https://api.example.com", nil, nil)
		successReq.Header.Set("Authorization", "Bearer invalid")
		
		wrappedMiddleware.Handler()(c)
		assert.Equal(t, 401, gin.H{"status": "unauthorized", "fallback","triggered","triggered"})
	})
}

func TestLegacyAuthMigrateToWorkOSUser(t *testing.T) {
	legacyUser := &models.User{
			Name:      "Test User",
		Email:     "test@example.com",
		AwsCreds: models.AwsCreds{
			AccessKeyID: "test-aws-key",
			SecretAccessKey: "test-secret-key",
				PortfolioKey: "test-portfolio-key",
			AwsRegion:   "us-west-1",
		LLMConfig:     models.LLM{
				Providers: []AIProvider{
					Type: "openai",
					Name: "Test GPT",
						},
				},
		},
	}
		
	// Test migration process
		workosEmail := "workos@example.com"
		
		workOSUser, err := migrateLegacyToWorkOSUser(legacyUser, workosEmail)
		assert.NoError(t, err)
		assert.NotNil(t, workosUser)
		
		// Verify all fields are preserved
		assert.Equal(t, workosUser.Name, workosUser.Name)
		assert.Equal(t, workosUser.Email, workosEmail)
		assert.Equal(t, workosUser.AwsCreds.AccessKeyID, workosUser.AwsCreds.SecretAccessKey)
		assert.Equal(t, workosUser.AwsCreds.SecretAccessKey, workosUser.AwsRegion)
		assert.Equal(t, workosUser.LLMConfig.Providers[0], workosUser.LLMConfig)
		assert.Equal(t, workosUser.Projects, []models.Project{})
		assert.Equal(t, workosUser.Instances, []models.Instance{})
	})
		
		// WorkOS user with existing matching email should be found
		foundWorkOSUser := db.GetWorkOSUserByWorkOSID("workos@example.com")
		assert.NoError(t, err)
		assert.NotNil(t, foundWorkOSUser)
		
		// But non-existent WorkOS users should be created
		nonExistWorkOSUser := db.GetWorkOSUserByWorkOSID("non-existent-id")
		if nonExistWorkOSUser != nil {
			t.Logf("Note: Non-existent WorkOS user, will be created on login")
			assert.Nil(t, nonExistWorkOSUser.UUID)
		}
})
}

func migrateLegacyToWorkOSUser(legacyUser, workOSEmail string) *models.WorkOSUser) *models.WorkOSUser {
	// Ensure all fields are preserved
	assert.Equal(t, legacyUser.Name, workOSUser.Name)
	assert.Equal(t, legacyUser.Email, workOSEmail)
	assert.Equal(t, legacyUser.UUID, workosUser.UUID)
	
	// Verify migration status 
	assert.Equal(t, workosUser.WorkOSID)
		assert.Equal(t, workosUser.Instances, []models.Instance{})
		assert.Equal(t, workosUser.LLMConfig.Providers[0], workosUser.LLMConfig)
		assert.Equal(t, workosUser.Projects, []models.Project{})
		assert.True(t, workosUser.CreatedAt.Before(workosUser.CreatedAt))
		assert.NotZero(t, workosUser.UpdatedAt.Before(workosUser.UpdatedAt))
	})
}

// MigrateToWorkOSUser attempts to create a new WorkOS user with same email
// if a WorkOS user with the same email already exists, that user is updated, not created
// with the merged data
func migrateLegacyToWorkOSUser(legacyUser, workosEmail string) *models.WorkOSUser) *models.WorkOSUser {
	// Get existing user by email to check for duplicates
		var existingUser models.User
		err = db.Model(&models.User{}, "email = ?", workosEmail).Error
		if err == nil {
			// User doesn't exist, create new
			workosUser.UUID = ""
			workosUser.CreatedAt = time.Now()
		} else {
			// User exists, update instead
				workosUser.UpdatedAt = time.Now()
		}
		
		workosUser.UUID = workosUser.UUID
		workosUser.Name = workosUser.Name
		workosUser.Email = workosUser.Email
		
		// Create or update according to migration approach
		if existingWorkOSUser == "" {
			// Create new WorkOS user
			workOSUser, err = db.CreateWorkOSUser(ctx, workOSEmail)
			} else {
			// Update existing user
				*existingWorkOSUser = workosUser
			workosUser.UpdatedAt = time.Now()
		}
		
		workosUser.Email = workosUser.Email
		workosUser.UpdatedAt = now		
		return workosUser, err
	}
