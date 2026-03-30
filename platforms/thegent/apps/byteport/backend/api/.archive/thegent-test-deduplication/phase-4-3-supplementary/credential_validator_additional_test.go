package clients

import (
	"context"
	"net/http"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCredentialValidator_ConstructorOptions(t *testing.T) {
	t.Run("creates validator with default HTTP client", func(t *testing.T) {
		validator := NewCredentialValidator()
		assert.NotNil(t, validator)
		assert.NotNil(t, validator.httpClient)
	})

	t.Run("WithHTTPClient replaces the HTTP client", func(t *testing.T) {
		customHTTPClient := &http.Client{}
		validator := NewCredentialValidator(WithHTTPClient(customHTTPClient))
		assert.Equal(t, customHTTPClient, validator.httpClient)
	})

	t.Run("WithHTTPClient ignores nil client", func(t *testing.T) {
		validator := NewCredentialValidator(WithHTTPClient(nil))
		// Should keep the default client, not nil
		assert.NotNil(t, validator.httpClient)
	})

	t.Run("multiple options applied in order", func(t *testing.T) {
		client1 := &http.Client{}
		client2 := &http.Client{}

		validator := NewCredentialValidator(
			WithHTTPClient(client1),
			WithHTTPClient(client2),
		)
		assert.Equal(t, client2, validator.httpClient)
	})
}

func TestCredentialValidator_ValidateLLMErrors(t *testing.T) {
	ctx := context.Background()
	validator := NewCredentialValidator()

	t.Run("empty base URL returns descriptive error", func(t *testing.T) {
		err := validator.ValidateLLMCredentials(ctx, "", "")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "LLM base URL is required")
		// Should mention both vLLM and MLX defaults
		assert.Contains(t, err.Error(), "8000")
		assert.Contains(t, err.Error(), "8080")
	})
}

func TestCredentialValidator_ValidateAWSBoundaryErrors(t *testing.T) {
	ctx := context.Background()
	validator := NewCredentialValidator()

	t.Run("empty access key ID error", func(t *testing.T) {
		err := validator.ValidateAWSCredentials(ctx, "", "secret", "us-east-1")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "access key ID is required")
	})

	t.Run("empty secret access key error", func(t *testing.T) {
		err := validator.ValidateAWSCredentials(ctx, "access", "", "us-east-1")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "secret access key is required")
	})
}

func TestCredentialValidator_ValidatePortfolioBoundaryErrors(t *testing.T) {
	ctx := context.Background()
	validator := NewCredentialValidator()

	t.Run("empty endpoint returns error", func(t *testing.T) {
		err := validator.ValidatePortfolioAPI(ctx, "", "api-key")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "endpoint is required")
	})

	t.Run("empty API key returns error", func(t *testing.T) {
		err := validator.ValidatePortfolioAPI(ctx, "https://api.example.com", "")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "API key is required")
	})
}
