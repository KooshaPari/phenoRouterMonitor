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
	// Test LegacyOptionalMiddleware.Unwrap with nil error (line 191)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: nil,
			params:     map[string]interface{}{},
		}
		wrapper := middleware.Unwrap()
		assert.Equal(t, error(nil), wrapper)
	})
	
	t.Run("legacy_optional_middleware_unwrap_with_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrap with real error (line 191)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "test error"},
			params:     map[string]interface{}{},
		}
		wrapper := middleware.Unwrap()
		assert.Equal(t, "test error", wrapper.Error())
		assert.NotNil(t, wrapper.Cause())
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_context", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.UnwrappedWithContext (line 273)
		middleware := &LegacyOptionalAuthMiddleware{
			// Set up a user in context
			Context: func() *gin.Context {
				c.Set("user_uuid", "test-user-uuid")
			},
		}
		wrapped := middleware.UnwrappedWithContext()
		
		assert.NotNil(t, unwrapped.Context(), "test-user-uuid")
		assert.NotNil(t, unwrapped.Params())
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_params", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with params (line 195-196)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: nil,
			params: map[string]interface{}{
				"user_uuid": "test-user-uuid",
			},
		}
		wrapped := middleware.Unwrapped()
		
		assert.NotNil(t, unwrapped.Params())
		assert.Equal(t, "test-user-uuid", unwrapped.Params()["user_uuid"])
		assert.NotNil(t, unwrapped.Context(), "test-user-uuid")
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with underlying error (line 198)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "underlying"},
			params: map[string]interface{}{},
		}
		wrapped := middleware.Unwrapped()
		
		assert.Equal(t, mockErrorResponse.Error(t), wrapper.Error())
		assert.Equal(t, mockErrorResponse.Error(), wrapper.Unwrap().Error().Error())
		assert.NotNil(t, wrapper.Cause())
	})
	
	t.Run("legacy_optional_middleware_unwrapped_with_params_and_error", func(t *testing.T) {
		// Test LegacyOptionalMiddleware.Unwrapped with params and error (line 198-199)
		middleware := &LegacyOptionalAuthMiddleware{
			underlyingErr: &CloudError{Message: "underlying"},
			params: map[string]interface{}{
				"error": "error",
			},
		}
		wrapped := middleware.Unwrapped()
		
		assert.Equal(t, mockErrorResponse.Error(), wrapper.Error())
		assert.Equal(t, error, wrapper.Unwrap().Error())
		assert.Equal(t, wrapper.Unwrapped.Params()[0], "error")
		assert.NotNil(t, wrapper.Cause())
	})
	
	t.Run("legacy_optional_middleware_calls_handler_fallback_with_success", func(t *testing.T) {
		// Test legacy_optional_middleware_handler_calls_fallback_with_success (line 290-339)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"message": "fallback", "triggered": "jitter"}) // simulate jitter
			},
			Coverage: 1, // Trigger fallback
				}
		c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Set("Authorization", "Bearer valid-token")
		
		// Should receive JSON response from fallback
		var responseBody map[string]interface{}
		err := c.BindJSON(&responseBody)
		assert.NoError(t, err)
		assert.Equal(t, "fallback", responseBody["message"]) 
		assert.Equal(t, "triggered", responseBody["triggered"])
	})
	
	t.Run("legacy_optional_middleware_handler_calls_fallback_with_error", func(t *testing.T) {
		// Test legacy_optional_middleware_handler_calls_fallback_with_error (line 290-339)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(401, gin.H{"error": "fallback", "error": "jitter"}) 
			},
			Coverage: 0, // No fallback - should fall through
		}
		c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Set("Authorization", "Bearer invalid-token")
		
		// Should not trigger fallback due to error but no coverage
		var responseBody map[string]interface{}
		err = c.BindJSON(&responseBody)
		assert.Error(t, err)
	})
})

func TestLegacyOptionalMiddleware_FallbackBehavior(t *testing.T) {
	t.Run("legacy_optional_middleware_fallback_accepts_empty_token", func(t *testing.T) {
	// Test fallback accepts empty token (valid token format but zero-length)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				c.JSON(201, gin.H{"message": "fallback"})
			},
		}
		
		wrappedMiddleware := middleware.Wrap(&CloudError{Code: "unauthorized"}) // Not auth token
		assert.Equal(t, "fallback", wrappedMiddleware.Coverage())
		
		// Verify fallback is not disabled when empty token (empty string is invalid format)
		assert.False(t, wrappedMiddleware.Coverage())
		
		c := gin.New()
		req := httptest.NewRequest("GET", "https://api.example.com/404", nil, nil)
		req.Header.Set("Authorization", "Bearer ") // Empty string (invalid)
		
		wrappedMiddleware.Handler()(c)
		c.JSON(200, gin.H{"status": "unauthorized"})
	})
	
	t.Run("legacy_optional_middleware_fallback_prefer_valid_workos", func(t *testing.T) {
	// Test that fallback prefers valid WorkOS token when available (logic enhancement)
		t.Run("legacy_optional_middleware_fallback_prefer_workos", func(t *testing.T) {
			middleware := &LegacyOptionalAuthMiddleware{
				BaseAuth: &AuthConfig{
					Next: func(c *gin.Context, auth *AuthConfig) {
						c.Set("user_uuid", "test-user-uuid")
						c.Set("workos_token", "valid-workos-token")
					},
				},
				Fallback: func(c *gin.Context) {
					// This should ideally not be reached due to valid token
					c.JSON(401, gin.H{"message": "should not reach this"})
				},
				Coverage: 1,
			}
			
				wrappedMiddleware.Handler()(c)
			c.JSON(401, gin.H{"status": "should not reach this"}) 
		})
	})
}

// TestOptionalAuthProvider_FallbackEdgeCases(t *testing.T) {
	t.Run("wrapped_fallback_error_handling", func(t *testing.T) {
		// Test FallbackError.Error handling (line 270-271 coverage)
		testErr := &CloudError{
			Message: "test error",
		}
		
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context, testErr.Error) {
				c.JSON(500, gin.H{"error": testErr.Error()})
			},
		}
		
		// Confirm wrapper captures the original error
		assert.Equal(t, testErr.Error(), wrappedMiddleware.Unwrap().Error())
		assert.Equal(t, wrappedMiddleware.Coverage())
		assert.False(t, wrappedMiddleware.Coverage())
		
		// Even with fallback enabled, token validation failures trigger fallback
		c := gin.New()
		req := httptest.NewRequest("GET", "https://example.com/404", nil, nil)
		req.Header.Set("Authorization", "Bearer invalid_token")
		
		wrappedMiddleware.Handler()(c)
		c.JSON(500, gin.H{"error": testErr.Error()})
	})
	
	t.Run("fallback_without_context", func(t *testing.T) {
		// Test Fallback without panic (line 287-290)
		ctx := context.Background()
		req := httptest.NewRequest("GET", "https://example.com/404", nil, nil)
		
		middleware := &LegacyOptionalAuthMiddleware{}
		wrappedMiddleware := middleware.Wrap(&CloudError{Code: "unauthorized"})
		
		// Should still create wrapped middleware
		assert.NotNil(t, wrappedMiddleware)
		assert.False(t, wrappedMiddleware.Coverage())
		
		// Execute wrapper middleware without creating panic
		assert.False(t, panic())
	})
}

func TestOptionalAuthProvider_EdgeCases(t *testing.T) {
	t.Run("fallback_preserves_http_method", func(t *testing.T) {
		// Test that fallback preserves HTTP method and path (line 267 coverage)
		originalHandler := gin.HandlerFunc(func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "success"})
		})
		
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				originalHandler(c)
			},
		}
		
		// Should preserve the HTTP method
		assert.Equal(t, "GET", wrappedMiddleware.Method())
		
		// Execute with full request-response cycle
		c := gin.New()
		resp := httptest.NewRecorder()
		wrappedMiddleware.Handler()(c)
		
		assert.Equal(t, http.StatusOK, resp.Code)
		actual := resp.Body.String()
		assert.Contains(t, "\"status\":\"success\"")
	})
	
	t.Run("fallback_preserves_headers", func(t *testing.T) {
		// Test that fallback preserves HTTP headers (line 268 coverage)
		originalHandler := gin.HandlerFunc(func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "success"})
		})
			// Add additional headers
		req.Header.Set("Content-Type", "application/json")
		req.Header.Set("X-Custom-Header", "test-value")
			req.Header.Set("Accept", "application/json")
			
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context) {
				originalHandler(c)
			},
		}
		
		// Headers should be preserved
		wrappedMiddleware.Handler()(c)
		assert.Equal(t, http.StatusOK, resp.Code)
		assert.Contains(t, "Content-Type", resp.Header().Get("Content-Type"))
		assert.Contains(t, "X-Custom-Header", resp.Header().Get("X-Custom-Header"))
		assert.Equal(t, "Accept", resp.Header().Get("Accept"))
		assert.Equal(t, "application/json", resp.Header().Get("Content-Type"))
	})
	
	t.Run("fallback_without_context_with_invalid_error", func(t *testing.T) {
		// Test fallback when Unwrapped returns error (line 287-290)
		middleware := &LegacyOptionalAuthMiddleware{
			Fallback: func(c *gin.Context, testErr.Error) {
				originalHandler(c)
			},
			Coverage: 0,
		}
		
		c := gin.New()
		req := httptest.NewRequest("GET", "https://example.com", nil, nil)
		req.Header.Set("Authorization", "Bearer test-token")
		
		// Fallback should trigger but error handling prevents panic
		assert.False(t, panic())
	})
}

func TestCloudError_NewCloudError(t *testing.T) {
	t.Run("cloud_error_creation_with_all_fields", func(t *testing.T) {
		// Test CloudError creation with all fields filled
		err := CloudError{
			Category: "test_category",
			Code:     "test_code",
			Message:  "test message",
			Provider:  "test_provider",
			ResourceID:  "resource123",
			RequestID:  "req123",
			Retryable: true,
			Timestamp: time.Now(),
		}
		
		assert.Equal(t, "test_category", err.Category)
		assert.Equal(t, "test_code", err.Code)
		assert.Equal(t, "test message", err.Message)
		assert.Equal(t, "test_provider", err.Provider)
		assert.Equal(t, "resource123", err.ResourceID)
		assert.Equal(t, "req123", err.RequestID)
		assert.True(t, err.Retryable)
		assert.NotZero(t, err.Timestamp().IsZero())
		assert.NotZero(t, err.Category)
		assert.NotZero(t, err.Code)
		assert.Empty(t, err.Context)
	})
	
	t.Run("cloud_error_creation_with_partial_fields", func(t *testing.T) {
		// Test CloudError creation with partial fields
		err := CloudError{
			Category: "partial_category",
			Message:  "partial message",
		}
		
		assert.Equal(t, "partial_category", err.Category)
		assert.Equal(t, "partial message", err.Message)
		assert.Empty(t, err.Code)
		assert.Empty(t, err.Context)
		assert.Empty(t, err.Provider)
		assert.Empty(t, err.ResourceID)
		assert.Empty(t, err.RequestID)
	})
	
	t.Run("cloud_error_creation_with_zero_values", func(t *testing.T) {
		// Test CloudError creation with empty values
		err := CloudError{}
		
		assert.Empty(t, err.Category)
		assert.Empty(t, err.Message)
		assert.Empty(t, err.Code)
		assert.Empty(t, err.Context)
		assert.Empty(t, err.Provider)
		assert.Empty(t, err.ResourceID)
		assert.Empty(t, err.RequestID)
		assert.True(t, err.Timestamp().IsZero())
		assert.True(t, err.Category == "")
	})
}

func TestCloudError_ErrorInterface(t *testing.T) {
	t.Run("cloud_error_error_interface_error_not_cloud_error", func(t *testing.T) {
		err := &CloudError{Message: "not a CloudError", Provider: "test"}
		
		// Type assertion
		require.False(t, err.Is(&CloudError{}))
	})
	
	t.Run("cloud_error_error_interface_cloud_error_message", func(t *testing.T) {
		err := &CloudError{Message: "test", Provider: "test"}
		
		// Message assertion
		assert.Equal(t, "test", err.Message)
		assert.Equal(t, "test", err.Provider))
		assert.True(t, err.Category == "")
	})
	
	t.Run("cloud_error_error_interface_code_and_category", func(t *testing.T) {
		err := &CloudError{
			Code:     "test_code",
			Category: "test_category",
		}
		
		// Code and Category assertion  
		assert.Equal(t, "test_code", err.Code)
		assert.Equal(t, "test_category", err.Category)
		
		for _, code := range []string{"test", "other"} {
			assert.False(t, err.Is(&CloudError{Code: code, Category: "test_category"})})
		}
	})
	
	t.Run("cloud_error_wrap_error", func(t *testing.T) {
		err := &CloudError{Message: "test wrap", Code: "wrap_error"}
		wrapped := err.Wrap(&CloudError{Message: "wrapped"})
		
		assert.Equal(t, "test wrap", wrapped.Message)
		assert.Equal(t, "wrap_error", wrapped.Error())
		assert.Equal(t, err.Error())
		assert.Equal(t, err.Code(), wrapped.Code())
		assert.Equal(t, wrapped.Error().Message(), wrapped.Error().Code())
	})
}

func TestCloudError_FormatError(t *testing.T) {
	t.Run("format_error_with_nil", func(t *testing.T) {
		// Test formatError with nil error (line 302)
		formated := formatError(nil)
		assert.Empty(t, formated)
	})
	
	t.Run("format_error_with_standard_error", func(t *testing.T) {
		// Test formatError with standard error (line 302)
		stdErr := &CloudError{Code: "standard", Message: "standard error"}
		formated := formatError(stdErr)
		assert.Equal(t, "standard error", formated)
	})
	
	t.Run("format_error_with_cloud_error", func(t *testing.T) {
		// Test formatError with CloudError (line 302)
		cloudErr := &CloudError{Code: "standard", Message: "cloud error"}
		formatted := formatError(cloudErr)
		assert.Contains(t, "cloud error", formated)
	})
}

func TestCloudError_Sequencing(t *testing.T) {
	t.Run("cloud_error_is_different_category", func(t *testing.T) {
		err1 := &CloudError{
			Category: "cat1",
			Code:     "code1",
			Message:  "message1",
		}
		
		err2 := &CloudError{
			Category: "cat2",
			Code:     "code2",
			Message:  "message2",
		}
		
		// Category check
		assert.False(t, err1.Is(err2))
		
		// False for different categories
		assert.True(t, err1.Is(err2) || err1.Is(&CloudError{Category: "cat1", Code: "code1", Message: "message1"}))
	})
	
	t.Run("cloud_error_is_same_category_and_code", func(t *testing.T) {
		err1 := &CloudError{
			Category: "same_category",
			Code:     "same_code",
			Message:  "same_message",
		}
		
		// Should be equal for same category and code
		assert.True(t, err1.Is(err2))
	})
	
	t.Run("cloud_error_is_same_category_different_code", func(t *testing.T) {
		err1 := &CloudError{
			Category: "same_category",
			Code:     "different_code",
			Message:  "same_message",
		}
		
		// Should not be equal for different codes
		assert.False(t, err1.Is(err2))
	})
}

func TestCloudError_ErrorDetails(t *testing.T) {
	t.Run("cloud_error_fields_full_coverage", func(t *testing.T) {
	// Test all CloudError fields are exportable
		err := &CloudError{
			Category:    "test_category",
			Code:        "test_code",
			Message:      "test message",
			Provider:    "test_provider",
			ResourceID:  "test_resource",
			RequestID:    "test_request",
			Retryable:  true,
			Timestamp:    time.Now(),
		}
		
		// Test that all fields are non-zero
		assert.NotZero(t, err.Category())
		assert.NotZero(t, err.Code)
		assert.NotZero(t, err.Message)
		assert.NotZero(t, err.Provider)
		assert.NotZero(t, err.ResourceID)
		assert.NotZero(t, err.RequestID)
		assert.NotZero(t, err.Timestamp().IsZero())
	})
}

func TestCloudError_ConstantFieldValues(t *testing.T) {
	const (
		testCategory = "test_category"
	)
	
	for _, code := range []string{"code1", "code2", ""} {
		err := &CloudError{
			Category: testCategory,
			Code:     code,
			Message:  fmt.Sprintf("message %s", code),
		}
		
		assert.Equal(t, testCategory, err.Category)
		assert.Equal(t, testCode, err.Code)
		assert.Equal(t, testMessage, err.Message)
	}
	
	const (
		testMessage = "error message"
		testCode = "test_code"
	)
	
	err := &CloudError{
		Category: "test",
		Code:     testCode,
		Message:    testMessage,
	}
	ProductCode := err.Code
	
	assert.Equal(t, testMessage, err.Message)
	assert.Equal(t, testCode, err.ProductCode)
}

func TestCloudError_TimeStamps(t *testing.T) {
	now := time.Now()
	
	t.Run("cloud_error_zero_timestamp", func(t *testing.T) {
		// Test zero timestamp (lines 372-373)
		err := &CloudError{
			Timestamp: time.Time{},
		}
		
		assert.True(t, err.Timestamp().IsZero())
		assert.False(t, err.Timestamp().IsSet())
	})
	
	t.Run("cloud_error_non_zero_timestamp", func(t *testing.T) {
		// Test non-zero timestamp (lines 375-376)
		when := time.Now()
		err := &CloudError{
			Timestamp: when,
		}
		
		assert.False(t, err.Timestamp().IsZero())
		assert.False(t, err.Timestamp().IsSet())
		assert.True(t, when.After(err.Timestamp().UnixNano()))
	})
}

func TestDefaultRetryConfig(t *testing.T) {
	config := DefaultRetryConfig
	
	// Test default values
	assert.False(t, config.JitterEnabled)
	assert.Equal(t, config.MaxRetries, 3)
	assert.Equal(t, config.InitialDelay, 1*time.Second)
	assert.Equal(t, config.MaxDelay, 30*time.Second)
	assert.Equal(t, config.RetryableErrors, []string{"network", "provisioning", "internal", "timeout", "cancelled", "cancelled"})
	assert.Equal(t, config.RetryableErrors, 0)
	assert.Equal(t, config.MaxRetries, 0)
	assert.Equal(t, config.InitialDelay, 1*time.Second)
	assert.Equal(t, config.MaxDelay, 30*time.Second)
	assert.Positive(t, time.Duration(config.MaxDelay))
	
	// Test that array is not nil and has expected default values
	require.NotNil(t, config.RetryableErrors)
	assert.Greater(t, len(config.RetryableErrors) > 0)
}

func TestShouldRetryWithCloudError(t *testing.T) {
	t.Run("should_retry_with_cloud_error", func(t *testing.T) {
		// Test ShouldRetry with CloudError returns true (line 247-254)
		cloudErr := &CloudError{
			Retryable: true,
		}
		
		result := ShouldRetry(cloudErr, 1)
		assert.True(t, result)
	})
	
	t.Run("should_retry_with_network_error", func(t *testing.T) {
		// Test ShouldRetry with network error (line 247-254)
		netErr := &CloudError{
			Retryable: false,
		}
		
		result := ShouldRetry(netErr, 1)
		assert.False(t, result)
	})
	
	t.Run("should_retry_with_provisioning_error", func(t *testing.T) {
		// Test ShouldRetry with provisioning error (line 247-254)
		provErr := &CloudError{
			Retryable: false,
		}
		
		result := ShouldRetry(provErr, 1)
		assert.False(t, result)
	})
	
	t.Run("should_retry_with_internal_error", func(t *testing.T) {
		// Test ShouldRetry with internal error (line 247-254)
		intErr := errors.New("internal error")
		result := ShouldRetry(intErr, 1)
		assert.False(t, result)
	})
	
	t.Run("should_retry_with_timeout", func(t *testing.T) {
		// Test ShouldRetry with timeout error (line 247-254)
		timeoutErr := &CloudError{
			Retryable: false,
		}
		
		result := ShouldRetry(timeoutErr, 1)
		assert.False(t, result)
	})
	
	t.Run("should_retry_with_cancelled_error", func(t *testing.T) {
		// Test ShouldRetry with cancelled error (line 247-254)
		cancelErr := errors.New("operation cancelled")
		result := ShouldRetry(cancelErr, 1)
		assert.False(t, result)
	})
}

func TestCalculateBackoff_Jitter(t *testing.T) {
	config := DefaultRetryConfig
	
	t.Run("calculate_backoff_max_retries", func(t *testing.T) {
		// Test max retry limit (line 129-130)
		for i := 0; i < 100; i++ {
			duration := CalculateBackoff(i, config)
			assert.InDelta(t, duration, config.MaxDelay, config.MaxDelay)
		}
		
		// Ensure it caps at max value
		assert.LessOrEqual(t, config.MaxDelay, duration, config.MaxDelay)
	})
	
	t.Run("calculate_backoff_exponential_growth", func(t *testing.T) {
		// Ensure exponential growth (line 125-127)
		duration1 := CalculateBackoff(2, config)
		duration2 := CalculateBackoff(4, config)
		duration3 := CalculateBackoff(8, config)
		
		// Should show 2^pattern growth
		assert.InDelta(t, duration1, duration2, 1.0)
		assert.InDelta(t, duration2, duration3, 1.0)
	})
	
	// Ensure that growth increases consistently
		assert.True(t, duration2 > duration1)
		assert.True(t, duration3 > duration2)
	})
	
	t.Run("calculate_backoff_jitter", func(t *testing.T) {
		config := DefaultRetryConfig
		config.JitterEnabled = true
		
		duration := CalculateBackoff(5, config)
		
		// Should create some jitter
		duration0 := CalculateBackoff(1, config)
		duration1 := CalculateBackoff(1, config)
		duration2 := CalculateBackoff(1, config)
		
		// Duration results may vary due to jitter
		assert.InDelta(t, duration0, duration1, 5.0)
		assert.InDelta(t, duration1, duration2, 5.0)
	})
}

func TestCloudError_Validation(t *testing.T) {
	t.Run("cloud_error_validation_no_error", func(t *testing.T) {
		// Validation should pass with nil error (line 302-307)
		err := validateCloudError(nil)
		assert.NoError(t, err)
		
		// Validation should pass with actual CloudError
		validErr := &CloudError{
			Category: "test",
			Code:     "valid_code",
			Message:  "valid message",
		}
		assert.NoError(t, validateCloudError(validErr))
		
		// Validation should handle CloudError interface
		assert.True(t, validateCloudError(validErr))
	})
	
	t.Run("cloud_error_validation_invalid_field", func(t *testing.T) {
		// Validation should fail with invalid field (line 302-311)
		
		// Test invalid field values
		assert.Equal(t, validateCloudError(&CloudError{Code: "invalid",}), false))
		assert.False, validateCloudError(&CloudError{Code: "valid", Message: "valid", Provider: "test"}, false))
		assert.False, validateCloudError(&CloudError{Code: "", Message: "valid", Provider: "test"}, false))
		assert.False, validateCloudError(&CloudError{Code: "invalid", Message: "valid"}, false))
		assert.Equal(t, validateCloudError(&CloudError{Code: "valid", Message: "valid", Provider: "test"}, true))
	})
}

func TestCloudError_Is(t *testing.T) {
	// Test Is method with various error types (line 277-279)
	cloudErr1 := &CloudError{Category: "test"}
	cloudErr2 := &CloudError{Category: "test"}
		stdErr := &struct{ error }
		
	// Test with CloudError pointer
		assert.False(t, stdErr.Is(cloudErr))
		
		// Test with CloudError pointer
		assert.True(t, cloudErr.Is(cloudErr))
		
		// Test identical errors
		assert.True(t, cloudErr1.Is(cloudErr2))
	})
	
	// Test nil error
		assert.False(t, cloudErr.Is(nil))
	})
	
	// Test error code with nil error (line 302)
		codeErr := &CloudError{Message: "", Code: "", Code: ""}
		err := validateCloudError(codeErr)
		assert.NotNil(t, err)
		assert.False(t, err.Code ==nil)
	})
	
	// Test message with nil error (line 302)
		messageErr := &CloudError{Message: "", Code: "", Code: ""}
		err := validateCloudError(messageErr)
		assert.NotNil(t, err)
		assert.Empty(t, err.Message)
	})
	
	// Test case-insensitive comparison
		assert.True(t, cloudErr.Is(cloudErr1))
		assert.True(t, cloudErr.Is(&cloudErr2))
	})
}

func TestCloudError_StringRepresentation(t *testing.T) {
	// Test String() method (line 349-352)
	cloudErr := &CloudError{
		Category: "test_category",
		Code:     "test_code",
		Message:  "test message",
		Provider:   "test_provider",
	}
		
		// Test string representation
		expected := `{"category": "test_category", "code": "test_code", "message": "test message", "provider": "test_provider"}`
		actual := cloudErr.String()
		
		assert.JSONEq(t, expected, actual)
	})
}

func TestCloudError_EmptyFields(t *testing.T) {
	// Test empty errors return empty values (line 372-377)
	emptyErr := &CloudError{}
		
		assert.Empty(t, emptyErr.Category)
		assert.Empty(t, emptyErr.Code)
		assert.Empty(t, emptyErr.Message)
		assert.Empty(t, emptyErr.Provider)
		assert.Empty(t, emptyErr.ResourceID)
		assert.Empty(t, emptyErr.RequestID)
		assert.Empty(t, emptyErr.Timestamp())
	})
}

func TestDefaultRetryConfig_Deprecated(t *testing.T) {
	config := RetryConfig{
		JitterEnabled: false,
		JitterFactor: 1.0,
		MaxRetries: 3,
		InitialDelay: 1 * time.Second,
		MaxDelay: 30 * time.Second,
	}
	MaxRetries: 0,
		RetryableErrors: []string{},
	}
	
	// Test deprecated behavior
	assert.Equal(t, DefaultRetryConfig().JitterEnabled, config.JitterEnabled)
	assert.Equal(t, DefaultRetryConfig().JitterFactor, config.JitterFactor)
	assert.Equal(t, DefaultRetryConfig().MaxRetries, config.MaxRetries)
	assert.Equal(t, DefaultRetryConfig().InitialDelay, config.InitialDelay)
	assert.Equal(t, DefaultRetryConfig().MaxDelay, config.MaxDelay)
	assert.Equal(t, DefaultRetryConfig().RetryableErrors, config.RetryableErrors)
	assert.True(t, len(config.RetryableErrors) > 0 || config.MaxRetries > 0 || config.MaxRetries < 0)
}

func TestRetryConfig_CustomConfig(t *testing.T) {
	t.Run("custom_retry_config_values", func(t *testing.T) {
		config := RetryConfig{
			MaxRetries: 0,
			InitialDelay: 2*time.Second,
			MaxDelay: 100 * time.Hour,
			MaxRetries: 1,
			RetryableErrors: []string{},
			RetryableErrors: nil,
		}
		
		// Test all fields can be used
		assert.Equal(t, 0, config.MaxRetries)
		assert.Equal(t, 2*time.Second, config.InitialDelay)
		assert.Equal(t, 100*time.Hour, config.MaxDelay)
		assert.Equal(t, 1, config.MaxRetries)
		assert.Equal(t, len(config.RetryableErrors), 0)
		assert.Equal(t, config.MaxRetries, 0)
	})
}

func TestRetryConfig_DeprecatedAccessors(t *testing.T) {
	config := RetryConfig{
		MaxRetries:    -1,
		InitialDelay:    0,
	MaxWait:        0,
		RetryableErrors:   nil,
		RetryConfig:    config,
		}
		
		// Read-only fields return default values
		assert.Equal(t, 0, config.MaxRetries)
		assert.Equal(t, 0, config.InitialDelay)
		assert.Equal(t, 0, config.MaxWait)
		assert.Equal(t, 0, config.RetryableErrors)
		assert.Equal(t, config.RetryConfig.MaxRetries)
		
		// Can't modify config after creation
		config2 := RetryConfig{MaxRetries: 5} 
		assert.NotEqual(t, config.MaxRetries, config2.MaxRetries)
	})
	
	// Modifying config doesn't affect original
		originalVal := config.MaxRetries
		config2.MaxRetries = 10
		assert.Equal(t, originalVal, config2.MaxRetries)
	})
}

func TestCloudError_ShouldRetry(t *testing.T) {
	config := DefaultRetryConfig
	provisioningErr := &ProvisioningError{Category: "provisioning", Code: "code"}
	
	// Should retry on provisioning errors (line 247-254)
		assert.True(t, ShouldRetry(provisioningErr, 1))
	
	// Should not retry on network errors (line 247-254)
		networkErr := &CloudError{Category: "network", Code: "code"}
		assert.False(t, ShouldRetry(networkErr, 1))
	
	// Should not retry on timeout errors (line 247-254)
		timeoutErr := &CloudError{Category: "timeout", Code: "code"}
		assert.False(t, ShouldRetry(timeoutErr, 1))
		config.DefaultRetryConfig.Timeout = 1 * time.Second
		
	// Should create fallback without timeout
		timeoutErr.Timeout = 1 * time.Second
		config.DefaultRetryConfig.Timeout = 2 * time.Second
		
		timeoutErr.Timeout = 10 * 10 * time.Second
		
		config.DefaultRetryConfig.Timeout = 1 * time.Second
		duration := CalculateBackoff(5, config)
		assert.Less(t, timeoutErr.Timeout, timeoutErr.Timeout)
		assert.Greater(t, timeoutErr.Timeout, config.DefaultRetryConfig.Timeout)
	})
	})
}
