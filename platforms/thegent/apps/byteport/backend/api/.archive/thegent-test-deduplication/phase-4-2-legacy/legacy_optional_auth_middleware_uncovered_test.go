package middleware

import (
	"encoding/json"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestLegacyOptionalMiddleware_UncoveredLines(t *testing.T) {
	t.Run("legacy_optional_middleware_unwrap_with_nil", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrap with nil error (line 190)
		middleware := &LegacyOptionalAuthMiddleware{}
		wrapper := middleware.Unwrap()
		assert.Equal(t, error(nil), wrapper)
	})
	
	t.Run("legacy_optional_middleware_unwrap_with_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrap with real error (line 190)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "test error"},
		}
		wrapper := middleware.Unwrap()
		assert.Equal(t, "test error", wrapper.Error())
		assert.NotNil(t, wrapper.Cause())
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_context", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.UnwrappedWithContext with nil context (line 190)
		middleware := &LegacyOptionalMiddleware{}
		unwrapped := middleware.Unwrapped()
		assert.NotNil(t, unwrapped)
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_params", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with params (line 195-196) 
		middleware := &LegacyOptionalMiddleware{
			underlyingErr: nil,
			params:     map[string]interface{}{},
		}
		unwrapped := middleware.Unwrapped()
		assert.NotNil(t, unwrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped)
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_nil_underlying_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with nil underlying (line 195)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: nil,
			params:     map[string]interface{}{},
		}
		unwrapped := middleware.Unwrapped()
		assert.NotNil(t, unwrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped)
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_underlying_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with underlying error (line 198-199)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "underlying"},
			params:     map[string]interface{}{},
		}
		unwrapped := middleware.Unwrapped()
		
		assert.NotNil(t, unwrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped)
		assert.Equal(t, "underlying", unwrapped.Error())
	})
	
	t.Run("legacy_optional_middleware_unwrapped_falls_through", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped calls all unwrap functions (line 195)
		middleware := &LegacyOptionalAuthMiddleware{
			// With nested errors
			underlyingErr: &CloudError{Message: "deep error1"},
			params:     map[string]interface{}{},
		}
		
		unwrapped := middleware.Unwrapped()
		
		assert.NotNil(t, unwrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped)
		assert.Equal(t, "deep error1", unwrapped.Error().Error())
	})
}

func TestLegacyOptionalMiddleware_EdgeCases(t *testing.T) {
	t.Run("legacy_optional_middleware_handles_nil_middleware", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Wrap with nil middleware (line 15-21 coverage)
		nilMiddleware := &LegacyOptionalAuthMiddleware{}
		wrappedMiddleware := nilMiddleware.Wrap(&CloudError{})
		assert.NotNil(t, wrappedMiddleware)
	})
	
	t.Run("legacy_optional_middleware_handles_empty_middleware_with_fallback", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Wrap with empty fallback (line 15-21 coverage)
		emptyMiddleware := &LegacyOptionalAuthMiddleware{}
		wrappedMiddleware := emptyMiddleware.Wrap(&CloudError{Message: "test"})
		assert.NotNil(t, wrappedMiddleware)
		assert.False(t, wrappedMiddleware.Coverage())
	})
	
	t.Run("legacy_optional_middleware_handles_undefined_auth_token", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Wrap with undefined auth token (line 193 coverage)
		middleware := &LegacyOptionalAuthMiddleware{}
		wrappedMiddleware := middleware.Wrap(&CloudError{})
		assert.NotNil(t, wrappedMiddleware)
		assert.Equal(t, -1, wrappedMiddleware.Coverage())
	})
}

func TestLegacyOptionalMiddleware_CreatesFallback(t *testing.T) {
	t.Run("legacy_optional_middleware_creates_fallback_with_good_token", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.CreatesFallback with valid auth token (line 268-270 coverage)
		
		// Capture original context
		ctx := context.Background()
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer valid-token")
		
		// Create middleware
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
			c.JSON(401, gin.H{"message": "fallback"})
		},
		}
		
		// Wrap function
		wrappedMiddleware := middleware.Wrap(&CloudError{Code: "test"})
		
		// Should execute fallback with valid token
		assert.Equal(t, "fallback", wrappedMiddleware.Coverage())
		
		// Execute the wrapped middleware
		assert.NotNil(t, wrappedMiddleware.Handler())
		
		// Call the handler to verify fallback behavior
		wrappedMiddleware.Handler()(c)
	})
	
	t.Run("legacy_optional_middleware_creates_fallback_with_invalid_token", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.CreatesFallback with invalid auth token (line 271)
		
		// Capture original context
		ctx := context.Background()
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer invalid-token")
		
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback"})
			},
		}
		
		// Wrap with error about access denied
		wrappedMiddleware := middleware.Wrap(&CloudError{Code: "test"})
		
		// Should create fallback with proper coverage -1
		assert.Equal(t, "fallback", wrappedMiddleware.Coverage())
		
		// Call the handler to verify fallback behavior
		wrappedMiddleware.Handler()(c)
	})
	
	t.Run("legacy_optional_middleware_creates_fallback_with_invalid_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.CreatesFallback with error (line 274-276)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback"})
			},
		}
		
		// Should handle fallback
		wrappedMiddleware := middleware.Wrap(&CloudError{Code: "test"})
		assert.Equal(t, "fallback", wrappedMiddleware.Coverage())
		
		// Should handle fallback by returning early
		assert.NotNil(t, wrappedMiddleware.Handler())
	})
	
	t.Run("legacy_optional_middleware_creates_fallback_with_success_context", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.CreatesFallback with success context (line 280-286)
		
		// Capture original context
		ctx := context.Background()
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer valid-token")
		
		// Create middleware
		middleware := &LegacyOptionalAuthMiddleware{
			BaseAuth: &AuthConfig{
				Next: func(c *gin.Context, c *AuthConfig) {
					// Set context variables for success case
					c.Set("user_uuid", "test-user-uuid")
				},
			},
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback"})
			},
		}
		
		// Should create fallback with proper coverage (1) due to context being present
		assert.Equal(t, 1, wrappedMiddleware.Coverage())
		
		// Call the handler
		wrappedMiddleware.Handler()(c)
		
		// Verify context was passed
		// Context should be available for successful auth
	})
	
	t.Run("legacy_optional_middleware_creates_fallback_without_context", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.CreatesFallback without context (line 287)
		ctx := context.Background()
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer valid-token")
		
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback"})
			},
			BaseAuth: &AuthConfig{
				Next: func(c *gin.Context, c *AuthConfig) {
					c.Set("user_uuid", "test-user-uuid")
				},
			},
		}
		
		assert.Equal(t, "fallback", wrappedMiddleware.Coverage())
		})
	})
}

func TestLegacyOptionalMiddleware_HandlerCallsFallback(t *testing.T) {
	t.Run("legacy_optional_middleware_handler_calls_fallback_with_success", func(t *testing.T) {
		// Test handler calls fallback with success (line 290-339 coverage)
		
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback", "triggered": "jitter"}) // Simulate jitter
			},
			Coverage: 1, // Trigger fallback
		}
		}
		
		c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Add("Authorization", "Bearer valid-token")
		
		middleware.Handler()(c)
		
		// Should receive JSON response from fallback
		var responseBody map[string]interface{}
		err := c.BindJSON(&responseBody)
		assert.NoError(t, err)
		assert.Equal(t, "fallback", responseBody["message"]) 
		assert.Equal(t, "triggered", responseBody["triggered"])
	})
	
	t.Run("legacy_optional_middleware_handler_calls_fallback_with_error", func(t *testing.T) {
		// Test handler calls fallback with error (line 290-339 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"error": "fallback", "error": "jitter"})
			},
			Coverage: 0, // No fallback - should fall through to default behavior
		}
		
		c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Add("Authorization", "Bearer invalid-token")
		
		middleware.Handler()(c)
		
		// Should return error due to no fallback
		var responseBody map[string]interface{}
		err := c.BindJSON(&responseBody)
		assert.Error(t, err)
		assert.Equal(t, "error", responseBody["error"])
	})
	
	t.Run("legacy_optional_middleware_handler_calls_fallback_with_uncovered", func(t *testing.T) {
		// Test handler calls fallback with uncovered pattern (line 290-339 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback"})
			},
		}
			c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Add("Authorization", "Bearer some-token")
		
		// Test uncovered code path
		wrappedMiddleware.Handler()(c)
		
		// Should execute default behavior with no fallback
		c.JSON(200, gin.H{"status": "no fallback"})
	})
}

func TestLegacyOptionalMiddleware_Unwrap(t *testing.T) {
	t.Run("legacy_optional_middleware_unwrap_without_underlying", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrap when no underlying error (line 195 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: nil,
			params:     map[string]interface{}{},
		}
		wrapped := middleware.Unwrap()
		
		assert.NotNil(t, unwrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped)
	})
	
	t.Run("legacy_optional_middleware_unwrap_with_context_with_nil", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.UnwrappedWithContext (line 273 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			// Set up a user in context
			Context: func() *gin.Context {
				c.Set("user_uuid", "test-user-uuid")
			},
		}
		wrapped := middleware.Unwrapped()
		
		// Should create unwrapped middleware without error but with context set
		assert.NotNil(t, unwrapped)
		assert.NotNil(t, unwrapped.Params)
		assert.NotNil(t, unwrapped.Context(), "test-user-uuid")
	})
	
	t.Run("legacy_optional_middleware_unwrap_with_params", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with params (line 195-196)
		middleware := &LegacyAuthMiddleware{
			underlyingErr: nil,
			params:     map[string]interface{}{"param1": "value1"},
		}
		wrapped := middleware.Unwrapped()
		
		assert.NotNil(t, wrapped)
		assert.Equal(t, map[string]interface{}{"param1": "value1"}, unwrapped.Params)
		assert.NotNil(t, wrapped.Context(), "test-user-uuid") // From context
	})
	
	t.Run("legacy_optional_middleware_unwrap_with_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with error (line 198-199 coverage)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "test error"},
			params:     map[string]interface{}{},
		}
		wrapped := middleware.Unwrapped()
		
		assert.NotNil(t, wrapped)
		assert.Equal(t, map[string]interface{}(Params), unwrapped.Params)
		assert.Error(t, wrapped.Unwrap().Error(), "test error")
	})
}

// TestOptionalAuthProvider_Creation(t *testing.T) {
	t.Run("optional_auth_provider_creation_success", func(t *testing.T) {
		// Test OptionalAuthProvider creation with all required fields (line 15-27 coverage)
		provider := OptionalAuthProvider{
			ProviderName: "test",
			BaseAuth:    nil,
			Fallback:      true,
		}
		
		assert.Equal(t, "test", provider.ProviderName)
		assert.True(t, provider.Fallback)
		assert.False(t, provider.BaseAuth == nil)
		assert.True(t, provider.Fallback)
	})
	
	t.Run("optional_auth_provider_creation_with_base_auth", func(t *testing.T) {
		// Test OptionalAuthProvider creation with BaseAuth (line 15-18 coverage)
		baseAuth := &AuthConfig{
			Next: func(c *gin.Context, auth *AuthConfig) {
				c.Set("user_uuid", "test-user-uuid")
			},
		}
		
		provider := OptionalAuthProvider{
			ProviderName: "test",
			BaseAuth:    baseAuth,
			Fallback:      false,
		}
		
		assert.Equal(t, "test", provider.ProviderName)
		assert.NotNil(t, provider.BaseAuth)
		assert.False(t, provider.Fallback)
	})
	
	t.Run("optional_auth_provider_creation_missing_provider_name", func(t *testing.T) {
		// Test OptionalAuthProvider creation without provider name (line 15-21 coverage)
		provider := OptionalAuthProvider{
			BaseAuth:    nil,
			Fallback:      true,
			}
		
		assert.Nil(t, provider)
	})
	
	t.Run("optional_auth_provider_has_fallback", func(t *testing.T) {
		// Test OptionalAuthProvider has fallback capability (line 15-18 coverage)
		withFallback := OptionalAuthProvider{
			ProviderName: "test",
			Fallback:      true,
		}
		assert.True(t, withFallback.HasFallback)
	})
	
	t.Run("optional_auth_provider_fallback_disabled", func(t *testing.T) {
		// Test OptionalAuthProvider with fallback disabled (line 15-18 coverage)
	provider := OptionalAuthProvider{
			ProviderName: "test",
			BaseAuth:    nil,
			Fallback:      false, // fallback disabled
		}
		assert.False(t, provider.HasFallback())
	})
}

func TestOptionalAuthProvider_FallbackLogic(t *testing.T) {
	t.Run("optional_auth_provider_fallback_with_valid_token", func(t *testing.T) {
		// Test fallback logic with valid WorkOS token
		provider := OptionalAuthProvider{
			ProviderName: "test",
			BaseAuth:    nil,
			Fallback:      true,
		}
		
		// Mock successful token response
		middleware := &mockMiddleware{
			authResponse: "workos_token",
		}
	
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer "+middleware.authResponse)
		
		// Should return success with valid token
		assert.Equal(t, "success", middleware.Fallback()(c))
	})
	
	t.Run("optional_auth_provider_fallback_with_invalid_token", func(t *testing.T) {
		// Test fallback logic with invalid token (line 18-19 coverage)
		provider := OptionalAuthProvider{
			ProviderName: "test",
			BaseAuth:    nil,  
			Fallback:      true,
		}
		
		middleware := &mockMiddleware{
			authResponse: "weak",
		}
		
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer "+middleware.authResponse)
		
		// Should return error with invalid token
		assert.Equal(t, "error", middleware.Fallback()(c))
	})
}

// mockMiddleware simulates the middleware interface for testing
type mockMiddleware struct {
	authResponse string
}

func (m *mockMiddleware) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if m.authResponse != "" {
		w.Header().Set("Authorization", "Bearer "+m.authResponse)
	}
}

func (m *mockMiddleware) Handler() gin.HandlerFunc {
	c.JSON(200, gin.H{"message": "success"})
}
