package version

import (
	"fmt"

	"github.com/KooshaPari/pheno-cli/internal/adapters"
)

// Calculate returns the registry-specific versioned string for a given
// base version, target channel, pre-release increment, and registry.
//
// Examples:
//
//	Calculate("0.2.0", "alpha", 1, "npm")       → "0.2.0-alpha.1"
//	Calculate("0.2.0", "beta",  1, "pypi")      → "0.2.0b1"
//	Calculate("0.2.0", "canary", 3, "pypi")     → "0.2.0.dev3"
//	Calculate("1.0.0", "prod",  0, "crates.io") → "1.0.0"
//	Calculate("0.2.0", "alpha", 1, "go_proxy")  → "v0.2.0-alpha.1"
func Calculate(baseVersion string, channel adapters.Channel, increment int, registry adapters.Registry) (string, error) {
	if channel == adapters.ChannelProd {
		return prodVersion(baseVersion, registry), nil
	}

	switch registry {
	case adapters.RegistryNPM, adapters.RegistryCrates, adapters.RegistryHex:
		return semverPreRelease(baseVersion, channel, increment), nil
	case adapters.RegistryPyPI:
		return pypiPreRelease(baseVersion, channel, increment)
	case adapters.RegistryGo:
		return "v" + semverPreRelease(baseVersion, channel, increment), nil
	case adapters.RegistryZig:
		return "v" + semverPreRelease(baseVersion, channel, increment), nil
	case adapters.RegistryMojo:
		return "", fmt.Errorf("mojo: %w", adapters.ErrNotSupported)
	default:
		return "", fmt.Errorf("unknown registry: %s", registry)
	}
}

func prodVersion(base string, registry adapters.Registry) string {
	switch registry {
	case adapters.RegistryGo, adapters.RegistryZig:
		return "v" + base
	default:
		return base
	}
}

// semverPreRelease produces SemVer pre-release: "X.Y.Z-channel.N"
func semverPreRelease(base string, channel adapters.Channel, inc int) string {
	return fmt.Sprintf("%s-%s.%d", base, channel, inc)
}

// pypiPreRelease produces PEP 440 pre-release versions.
//
// Mapping:
//
//	alpha  → XaN     (e.g., 0.2.0a1)
//	canary → X.devN  (e.g., 0.2.0.dev1) — sorts before alpha in PEP 440
//	beta   → XbN     (e.g., 0.2.0b1)
//	rc     → XrcN    (e.g., 0.2.0rc1)
func pypiPreRelease(base string, channel adapters.Channel, inc int) (string, error) {
	switch channel {
	case adapters.ChannelAlpha:
		return fmt.Sprintf("%sa%d", base, inc), nil
	case adapters.ChannelCanary:
		return fmt.Sprintf("%s.dev%d", base, inc), nil
	case adapters.ChannelBeta:
		return fmt.Sprintf("%sb%d", base, inc), nil
	case adapters.ChannelRC:
		return fmt.Sprintf("%src%d", base, inc), nil
	default:
		return "", fmt.Errorf("unknown channel: %s", channel)
	}
}

// DistTag returns the npm dist-tag for a channel.
func DistTag(channel adapters.Channel) string {
	if channel == adapters.ChannelProd {
		return "latest"
	}
	return string(channel)
}
