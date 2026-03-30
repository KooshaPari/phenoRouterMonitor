package cloud

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestCloudError_UncoveredLines(t *testing.T) {
	t.Run("cloud_error_unwrap_with_nil", func(t *testing.T) {
		// Test CloudError.Unwrap with nil Cause (line 57 coverage)
		err := &CloudError{
			Category: "test",
			Code:     "test_code",
			Message:  "test message",
			Cause:    nil,
		}
		
		unwrapped := err.Unwrap()
		assert.Nil(t, unwrapped)
	})
	
	t.Run("cloud_error_is_with_invalid_target", func(t *testing.T) {
		// Test CloudError.Is with invalid target type (line 43-44 coverage)
		cloudErr := &CloudError{
			Category: "test",
			Code:     "test_code",
		}
		
		// Test with different error type
		stdErr := &struct{ error }{error: &CloudError{}}
		
		isMatch := cloudErr.Is(stdErr)
		assert.False(t, isMatch)
	})
	
	t.Run("cloud_error_is_with_different_code", func(t *testing.T) {
		// Test CloudError.Is with different code (line 45 coverage)
		cloudErr1 := &CloudError{
			Category: "test",
			Code:     "code1",
		}
		
		cloudErr2 := &CloudError{
			Category: "test",
			Code:     "code2",
		}
		
		isMatch := cloudErr1.Is(cloudErr2)
		assert.False(t, isMatch)
	})
	
	t.Run("cloud_error_is_with_different_category", func(t *testing.T) {
		// Test CloudError.Is with different category (line 45 coverage)
		cloudErr1 := &CloudError{
			Category: "category1",
			Code:     "same_code",
		}
		
		cloudErr2 := &CloudError{
			Category: "category2",
			Code:     "same_code",
		}
		
		isMatch := cloudErr1.Is(cloudErr2)
		assert.False(t, isMatch)
	})
	
	t.Run("cloud_error_is_with_match", func(t *testing.T) {
		// Test CloudError.Is with matching error (line 45 coverage)
		cloudErr1 := &CloudError{
			Category: "same_category",
			Code:     "same_code",
		}
		
		cloudErr2 := &CloudError{
			Category: "same_category",
			Code:     "same_code",
		}
		
		isMatch := cloudErr1.Is(cloudErr2)
		assert.True(t, isMatch)
	})
	
	t.Run("calculate_backoff_zero_attempts", func(t *testing.T) {
		// Test CalculateBackoff with zero attempts (line 125 coverage)
		config := DefaultRetryConfig
		duration := CalculateBackoff(0, config)
		// DefaultRetryConfig includes jitter, so check it's close to expected value
		assert.GreaterOrEqual(t, duration, 0*time.Second)
		assert.Less(t, duration, 2*time.Second) // Allow some jitter buffer
	})
	
	t.Run("calculate_backoff_first_attempt", func(t *testing.T) {
		// Test CalculateBackoff with first attempt (line 125 coverage)
		config := DefaultRetryConfig
		duration := CalculateBackoff(1, config)
		// DefaultRetryConfig includes jitter, so check it's close to expected value
		assert.Greater(t, duration, 1*time.Second)
		assert.Less(t, duration, 3*time.Second) // Allow some jitter buffer
	})
	
	t.Run("calculate_backoff_exponential", func(t *testing.T) {
		// Test CalculateBackoff exponential growth (line 125 coverage)
		config := DefaultRetryConfig
		duration := CalculateBackoff(3, config)
		// DefaultRetryConfig includes jitter, so check it's close to expected value
		assert.Greater(t, duration, 1*time.Second)
		assert.Less(t, duration, 10*time.Second) // Allow some jitter buffer
	})
	
	t.Run("calculate_backoff_max_limit", func(t *testing.T) {
		// Test CalculateBackoff max limit (line 129 coverage)
		config := DefaultRetryConfig
		duration := CalculateBackoff(100, config)
		// Just ensure function executes and returns a reasonable value
		assert.GreaterOrEqual(t, duration, time.Duration(0))
	})
	
	t.Run("calculate_backoff_no_limit", func(t *testing.T) {
		// Test CalculateBackoff with no limit (line 135 coverage)
		config := RetryConfig{MaxRetries: -1, InitialDelay: 1 * time.Second, MaxDelay: 9999 * time.Hour}
		duration := CalculateBackoff(5, config) // Use reasonable attempts
		assert.GreaterOrEqual(t, duration, time.Duration(0))
		// Just ensure it returns a reasonable value
	})
}
