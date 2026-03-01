package version

import (
	"testing"

	"github.com/KooshaPari/pheno-cli/internal/adapters"
)

func TestCalculate(t *testing.T) {
	tests := []struct {
		name      string
		base      string
		channel   adapters.Channel
		increment int
		registry  adapters.Registry
		want      string
		wantErr   bool
	}{
		// NPM
		{"npm alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryNPM, "0.2.0-alpha.1", false},
		{"npm beta", "0.2.0", adapters.ChannelBeta, 2, adapters.RegistryNPM, "0.2.0-beta.2", false},
		{"npm rc", "1.0.0", adapters.ChannelRC, 1, adapters.RegistryNPM, "1.0.0-rc.1", false},
		{"npm canary", "0.2.0", adapters.ChannelCanary, 3, adapters.RegistryNPM, "0.2.0-canary.3", false},
		{"npm prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryNPM, "1.0.0", false},

		// PyPI
		{"pypi alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryPyPI, "0.2.0a1", false},
		{"pypi beta", "0.2.0", adapters.ChannelBeta, 1, adapters.RegistryPyPI, "0.2.0b1", false},
		{"pypi rc", "1.0.0", adapters.ChannelRC, 2, adapters.RegistryPyPI, "1.0.0rc2", false},
		{"pypi canary", "0.2.0", adapters.ChannelCanary, 3, adapters.RegistryPyPI, "0.2.0.dev3", false},
		{"pypi prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryPyPI, "1.0.0", false},

		// Crates.io
		{"crates alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryCrates, "0.2.0-alpha.1", false},
		{"crates prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryCrates, "1.0.0", false},

		// Go
		{"go alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryGo, "v0.2.0-alpha.1", false},
		{"go prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryGo, "v1.0.0", false},

		// Hex
		{"hex beta", "0.2.0", adapters.ChannelBeta, 1, adapters.RegistryHex, "0.2.0-beta.1", false},
		{"hex prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryHex, "1.0.0", false},

		// Zig
		{"zig alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryZig, "v0.2.0-alpha.1", false},
		{"zig prod", "1.0.0", adapters.ChannelProd, 0, adapters.RegistryZig, "v1.0.0", false},

		// Mojo — not supported
		{"mojo alpha", "0.2.0", adapters.ChannelAlpha, 1, adapters.RegistryMojo, "", true},

		// Unknown registry
		{"unknown registry", "1.0.0", adapters.ChannelAlpha, 1, adapters.Registry("nope"), "", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := Calculate(tt.base, tt.channel, tt.increment, tt.registry)
			if (err != nil) != tt.wantErr {
				t.Fatalf("Calculate() error = %v, wantErr %v", err, tt.wantErr)
			}
			if got != tt.want {
				t.Errorf("Calculate() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestDistTag(t *testing.T) {
	tests := []struct {
		channel adapters.Channel
		want    string
	}{
		{adapters.ChannelProd, "latest"},
		{adapters.ChannelAlpha, "alpha"},
		{adapters.ChannelBeta, "beta"},
		{adapters.ChannelCanary, "canary"},
		{adapters.ChannelRC, "rc"},
	}
	for _, tt := range tests {
		t.Run(string(tt.channel), func(t *testing.T) {
			if got := DistTag(tt.channel); got != tt.want {
				t.Errorf("DistTag(%q) = %q, want %q", tt.channel, got, tt.want)
			}
		})
	}
}
