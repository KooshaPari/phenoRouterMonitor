// Policy Engine Module
// wraps: phenotype-policy-engine (Rust crate) — Rule-based policy evaluation

// ============================================================================
// CORE INTERFACES
// ============================================================================

/**
 * Rule abstraction for policy evaluation
 */
export interface Rule<TContext = unknown> {
  readonly id: string;
  readonly name: string;
  readonly priority?: number;
  evaluate(context: TContext): Promise<RuleResult>;
}

/**
 * Result of evaluating a rule
 */
export interface RuleResult {
  allowed: boolean;
  reason?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Policy result from engine evaluation
 */
export interface PolicyResult {
  allowed: boolean;
  reason?: string;
  evaluatedRules: string[];
  metadata?: Record<string, unknown>;
}

/**
 * Policy engine for evaluating rules against contexts
 */
export interface PolicyEngine<TContext = unknown> {
  /**
   * Register a rule
   */
  addRule(rule: Rule<TContext>): void;

  /**
   * Remove a rule by ID
   */
  removeRule(ruleId: string): void;

  /**
   * Evaluate all rules against a context
   * Returns true if ANY rule allows (OR logic)
   */
  evaluate(context: TContext, mode?: 'any' | 'all'): Promise<PolicyResult>;

  /**
   * Get all registered rules
   */
  getRules(): Rule<TContext>[];

  /**
   * Clear all rules
   */
  clear(): void;
}

/**
 * Policy configuration
 */
export interface PolicyConfig {
  mode?: 'any' | 'all'; // 'any' = OR, 'all' = AND (default: 'any')
  stopOnFirstMatch?: boolean;
  enableCaching?: boolean;
  cacheTtlMs?: number;
}

/**
 * Conditional rule with guard expressions
 */
export interface ConditionalRule<TContext> extends Rule<TContext> {
  guard: (context: TContext) => boolean;
  action: (context: TContext) => Promise<RuleResult>;
}

// ============================================================================
// BUILT-IN RULES
// ============================================================================

/**
 * Always-allow rule
 */
export class AllowRule<TContext = unknown> implements Rule<TContext> {
  id = 'allow-all';
  name = 'Allow All';

  async evaluate(_context: TContext): Promise<RuleResult> {
    return { allowed: true };
  }
}

/**
 * Always-deny rule
 */
export class DenyRule<TContext = unknown> implements Rule<TContext> {
  id = 'deny-all';
  name = 'Deny All';

  async evaluate(_context: TContext): Promise<RuleResult> {
    return { allowed: false, reason: 'Denied by policy' };
  }
}

/**
 * Conditional rule implementation
 */
export class SimpleConditionalRule<TContext> implements ConditionalRule<TContext> {
  constructor(
    readonly id: string,
    readonly name: string,
    readonly guard: (context: TContext) => boolean,
    readonly action: (context: TContext) => Promise<RuleResult>,
    readonly priority?: number,
  ) {}

  async evaluate(context: TContext): Promise<RuleResult> {
    if (!this.guard(context)) {
      return { allowed: false, reason: `Guard failed for rule ${this.id}` };
    }
    return this.action(context);
  }
}

// ============================================================================
// POLICY ENGINE IMPLEMENTATION
// ============================================================================

/**
 * In-memory policy engine implementation
 */
export class PolicyEngineImpl<TContext = unknown> implements PolicyEngine<TContext> {
  private rules: Map<string, Rule<TContext>> = new Map();
  private config: PolicyConfig;

  constructor(config: PolicyConfig = {}) {
    this.config = {
      mode: config.mode ?? 'any',
      stopOnFirstMatch: config.stopOnFirstMatch ?? false,
      enableCaching: config.enableCaching ?? false,
      cacheTtlMs: config.cacheTtlMs ?? 60000,
    };
  }

  addRule(rule: Rule<TContext>): void {
    this.rules.set(rule.id, rule);
  }

  removeRule(ruleId: string): void {
    this.rules.delete(ruleId);
  }

  async evaluate(context: TContext, mode?: 'any' | 'all'): Promise<PolicyResult> {
    const evaluationMode = mode ?? this.config.mode ?? 'any';
    const evaluatedRules: string[] = [];
    let allowed = evaluationMode === 'all'; // Default for 'all' is true until proven false

    for (const [, rule] of this.rules) {
      const result = await rule.evaluate(context);
      evaluatedRules.push(rule.id);

      if (evaluationMode === 'any') {
        // OR logic
        if (result.allowed) {
          allowed = true;
          if (this.config.stopOnFirstMatch) {
            break;
          }
        }
      } else {
        // AND logic
        if (!result.allowed) {
          allowed = false;
          if (this.config.stopOnFirstMatch) {
            break;
          }
        }
      }
    }

    return {
      allowed,
      evaluatedRules,
      metadata: {
        evaluationTime: new Date().toISOString(),
        ruleCount: this.rules.size,
      },
    };
  }

  getRules(): Rule<TContext>[] {
    return Array.from(this.rules.values());
  }

  clear(): void {
    this.rules.clear();
  }
}

// ============================================================================
// UTILITIES
// ============================================================================

/**
 * Helper to create a conditional rule
 */
export function createConditionalRule<TContext>(
  id: string,
  name: string,
  guard: (context: TContext) => boolean,
  action: (context: TContext) => Promise<RuleResult>,
  priority?: number,
): ConditionalRule<TContext> {
  return new SimpleConditionalRule(id, name, guard, action, priority);
}

/**
 * Helper to create a permission rule (checks for a specific permission)
 */
export function createPermissionRule<TContext extends { permissions?: string[] }>(
  requiredPermission: string,
): Rule<TContext> {
  return {
    id: `permission-${requiredPermission}`,
    name: `Require ${requiredPermission}`,
    evaluate: async (context) => {
      const hasPermission = context.permissions?.includes(requiredPermission);
      return {
        allowed: hasPermission ?? false,
        reason: hasPermission ? undefined : `Missing permission: ${requiredPermission}`,
      };
    },
  };
}
