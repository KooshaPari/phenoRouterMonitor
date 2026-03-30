package models

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDeploymentIsTerminatedByTimestamp(t *testing.T) {
	t.Run("terminated when timestamp present regardless of status", func(t *testing.T) {
		now := time.Now().UTC().Add(-30 * time.Second)
		deployment := &Deployment{
			Status:       "deploying",
			TerminatedAt: &now,
		}

		assert.True(t, deployment.IsTerminated(), "deployment should be treated as terminated when TerminatedAt is set")
	})

	t.Run("clears termination metadata when status changes", func(t *testing.T) {
		deployment := &Deployment{
			Status: "terminated",
		}

		deployment.SetStatus("terminated")
		require.NotNil(t, deployment.TerminatedAt)

		// move back to deploying to ensure helper does not leave stale timestamps
		deployment.SetStatus("deploying")
		assert.Nil(t, deployment.TerminatedAt)
	})
}

func TestHostDeploymentSetStatusUpdatesTimestamp(t *testing.T) {
	hostDeployment := &HostDeployment{
		Status:    "pending",
		UpdatedAt: time.Now().Add(-1 * time.Hour),
	}

	hostDeployment.SetStatus("running")

	assert.Equal(t, "running", hostDeployment.Status)
	assert.True(t, time.Since(hostDeployment.UpdatedAt) < time.Second, "UpdatedAt should be refreshed to a recent timestamp")
}

func TestAPIRateLimitIncrementCount(t *testing.T) {
	rateLimit := &APIRateLimit{
		RequestCount: 0,
		WindowStart:  time.Now().UTC(),
	}

	for i := 0; i < 3; i++ {
		rateLimit.IncrementCount()
	}

	assert.Equal(t, 3, rateLimit.RequestCount)
	assert.False(t, rateLimit.IsExpired(), "recent window should not be considered expired")
}
