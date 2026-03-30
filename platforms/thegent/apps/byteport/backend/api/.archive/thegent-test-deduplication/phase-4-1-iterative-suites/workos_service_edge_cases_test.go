package auth

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/rsa"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/go-jose/go-jose/v4"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type roundTripFunc func(*http.Request) (*http.Response, error)

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req)
}

func TestWorkOSAuthService_ValidateJWTToken_EdgeCases(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	t.Run("handles empty token", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		assert.Contains(t, err.Error(), "token is empty")
	})

	t.Run("handles whitespace-only token", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "   \t\n  ")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		assert.Contains(t, err.Error(), "token is empty")
	})

	t.Run("handles invalid JWT format", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "invalid.jwt.token")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		assert.Contains(t, err.Error(), "failed to parse JWT")
	})

	t.Run("handles malformed JWT", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "not-a-jwt")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		assert.Contains(t, err.Error(), "failed to parse JWT")
	})

	t.Run("handles test token with minimal format", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "test-user123")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "user123", userInfo.ID)
		assert.Equal(t, "test@example.com", userInfo.Email)
		assert.Equal(t, "Test", userInfo.FirstName)
		assert.Equal(t, "User", userInfo.LastName)
	})

	t.Run("handles test token with email", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "test-user456-john_at_example.com")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "user456", userInfo.ID)
		assert.Equal(t, "john@example.com", userInfo.Email)
		assert.Equal(t, "Test", userInfo.FirstName)
		assert.Equal(t, "User", userInfo.LastName)
	})

	t.Run("handles mock token", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "mock-user789")
		require.NoError(t, err)
		assert.NotNil(t, userInfo)
		assert.Equal(t, "user789", userInfo.ID)
		assert.Equal(t, "test@example.com", userInfo.Email)
	})

	t.Run("handles invalid test token format", func(t *testing.T) {
		userInfo, err := service.ValidateToken(ctx, "test")
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		// The error could be either "invalid test token format" or JWT parsing error
		assert.True(t, strings.Contains(err.Error(), "invalid test token format") ||
			strings.Contains(err.Error(), "failed to parse JWT"))
	})
}

func TestWorkOSAuthService_GetWorkOSPublicKey(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("handles successful JWKS fetch", func(t *testing.T) {
		original := httpGet
		defer func() { httpGet = original }()

		jwks := JWKSResponse{
			Keys: []jose.JSONWebKey{
				{
					KeyID: "test-key-1",
					Key:   generateTestRSAPublicKey(),
				},
			},
		}
		body, _ := json.Marshal(jwks)

		var capturedURL string
		httpGet = func(url string) (*http.Response, error) {
			capturedURL = url
			return &http.Response{
				StatusCode: http.StatusOK,
				Header:     make(http.Header),
				Body:       io.NopCloser(bytes.NewReader(body)),
			}, nil
		}

		key, err := service.getWorkOSPublicKey(ctx, "test-key-1")
		assert.NoError(t, err)
		assert.NotNil(t, key)
		assert.Equal(t, "https://api.workos.com/.well-known/jwks.json", capturedURL)
	})

	t.Run("handles JWKS fetch failure", func(t *testing.T) {
		original := httpGet
		defer func() { httpGet = original }()

		httpGet = func(url string) (*http.Response, error) {
			return nil, fmt.Errorf("network error")
		}

		_, err := service.getWorkOSPublicKey(ctx, "test-key")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to fetch JWKS")
	})
}

func TestWorkOSAuthService_ExchangeWithWorkOS(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("handles successful token exchange", func(t *testing.T) {
		originalFactory := httpClientFactory
		defer func() { httpClientFactory = originalFactory }()

		httpClientFactory = func() *http.Client {
			return &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
				assert.Equal(t, "https://api.workos.com/user_management/authenticate", req.URL.String())
				assert.Equal(t, "POST", req.Method)
				body, _ := io.ReadAll(req.Body)
				var payload map[string]string
				require.NoError(t, json.Unmarshal(body, &payload))
				assert.Equal(t, "test-client-id", payload["client_id"])

				response := map[string]interface{}{
					"access_token": "workos-access-token",
					"id_token":     "workos-id-token",
					"expires_in":   3600,
					"token_type":   "Bearer",
				}
				respBody, _ := json.Marshal(response)
				return &http.Response{
					StatusCode: http.StatusOK,
					Header:     make(http.Header),
					Body:       io.NopCloser(bytes.NewReader(respBody)),
				}, nil
			})}
		}

		resp, err := service.exchangeWithWorkOS(ctx, "test-code", "test-client-id", "test-client-secret")
		assert.NoError(t, err)
		assert.Equal(t, "workos-access-token", resp.AccessToken)
	})

	t.Run("handles invalid JSON payload", func(t *testing.T) {
		originalFactory := httpClientFactory
		defer func() { httpClientFactory = originalFactory }()

		httpClientFactory = func() *http.Client {
			return &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
				return nil, fmt.Errorf("request error")
			})}
		}

		_, err := service.exchangeWithWorkOS(ctx, "code", "client-id", "client-secret")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to make request")
	})
}

func TestWorkOSAuthService_ExchangeCodeForToken_EdgeCases(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	t.Run("handles empty code", func(t *testing.T) {
		tokenResp, err := service.ExchangeCodeForToken(ctx, "")
		assert.Error(t, err)
		assert.Nil(t, tokenResp)
		assert.Contains(t, err.Error(), "authorization code is required")
	})

	t.Run("handles whitespace code", func(t *testing.T) {
		tokenResp, err := service.ExchangeCodeForToken(ctx, "   \t\n  ")
		assert.Error(t, err)
		assert.Nil(t, tokenResp)
		assert.Contains(t, err.Error(), "authorization code is required")
	})

	t.Run("handles uninitialized service", func(t *testing.T) {
		uninitializedService := NewWorkOSAuthService(manager)
		tokenResp, err := uninitializedService.ExchangeCodeForToken(ctx, "test-code")
		assert.Error(t, err)
		assert.Nil(t, tokenResp)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})
}

func TestWorkOSAuthService_ValidateJWTToken_RealJWT(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	t.Run("handles JWT with missing kid", func(t *testing.T) {
		// Create a JWT without kid in header
		token := "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJpYXQiOjE2MzQ1Njc4OTAsImV4cCI6OTk5OTk5OTk5OX0.invalid-signature"

		userInfo, err := service.ValidateToken(ctx, token)
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		// Should fail at JWT parsing or public key fetching
	})

	t.Run("handles JWT with invalid signature", func(t *testing.T) {
		// Create a JWT with invalid signature
		token := "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InRlc3Qta2V5In0.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJpYXQiOjE2MzQ1Njc4OTAsImV4cCI6OTk5OTk5OTk5OX0.invalid-signature"

		userInfo, err := service.ValidateToken(ctx, token)
		assert.Error(t, err)
		assert.Nil(t, userInfo)
		// Should fail at JWT validation
	})
}

func TestWorkOSAuthService_GetAuthURL_EdgeCases(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()

	t.Run("handles uninitialized service", func(t *testing.T) {
		url, err := service.GetAuthURL(ctx, "test-state")
		assert.Error(t, err)
		assert.Empty(t, url)
		assert.Contains(t, err.Error(), "WorkOS client not initialized")
	})

	t.Run("handles empty state", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

		url, err := service.GetAuthURL(ctx, "")
		require.NoError(t, err)
		assert.NotEmpty(t, url)
		assert.Contains(t, url, "state=")
	})

	t.Run("handles special characters in state", func(t *testing.T) {
		require.NoError(t, service.Initialize(ctx))

		state := "test-state-with-special-chars!@#$%^&*()"
		url, err := service.GetAuthURL(ctx, state)
		require.NoError(t, err)
		assert.NotEmpty(t, url)
		assert.Contains(t, url, state)
	})
}

func TestWorkOSAuthService_Middleware_EdgeCases(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	gin.SetMode(gin.TestMode)

	t.Run("handles malformed authorization header", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "InvalidFormat token")
		c.Request = req

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})

	t.Run("handles authorization header with extra spaces", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "  Bearer  test-token  ")
		c.Request = req

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})

	t.Run("handles empty bearer token", func(t *testing.T) {
		w := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(w)
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer ")
		c.Request = req

		middleware := service.Middleware()
		middleware(c)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
		assert.True(t, c.IsAborted())
	})
}

func TestWorkOSAuthService_OptionalMiddleware_EdgeCases(t *testing.T) {
	manager := setupMockSecretsManager()
	service := NewWorkOSAuthService(manager)
	ctx := context.Background()
	require.NoError(t, service.Initialize(ctx))

	gin.SetMode(gin.TestMode)

	t.Run("handles malformed authorization header gracefully", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "InvalidFormat token")
		router.ServeHTTP(w, req)

		// Should not abort, just continue without user context
		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("handles empty authorization header", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		// No Authorization header
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("handles authorization header with only Bearer", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, router := gin.CreateTestContext(w)

		router.Use(service.OptionalMiddleware())
		router.GET("/test", func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "ok"})
		})

		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Authorization", "Bearer")
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})
}

// Helper function to generate a test RSA public key
func generateTestRSAPublicKey() *rsa.PublicKey {
	// Generate a test RSA key pair for testing
	// Note: In a real implementation, you'd use a proper key generation
	// For testing purposes, we'll create a minimal key
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil
	}
	return &key.PublicKey
}
