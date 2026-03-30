// State Machine — Finite state machines with guards and transitions
// wraps: phenotype-state-machine Rust crate

export interface State<TId = string> {
  readonly id: TId;
  readonly name: string;
  readonly metadata?: Record<string, unknown>;
}

export interface Transition<TStateId = string, TContext = unknown> {
  readonly from: TStateId;
  readonly to: TStateId;
  readonly event: string;
  readonly guard?: (context: TContext) => boolean;
  readonly action?: (context: TContext) => TContext;
}

export interface StateMachineConfig<TStateId = string, TContext = unknown> {
  readonly initialState: TStateId;
  readonly states: State<TStateId>[];
  readonly transitions: Transition<TStateId, TContext>[];
}

export class StateMachine<TStateId = string, TContext = unknown> {
  private currentState: TStateId;
  private context: TContext;

  constructor(
    private readonly config: StateMachineConfig<TStateId, TContext>,
    initialContext: TContext,
  ) {
    this.currentState = config.initialState;
    this.context = initialContext;
  }

  get state(): TStateId {
    return this.currentState;
  }

  get currentContext(): TContext {
    return this.context;
  }

  canTransition(event: string): boolean {
    return this.config.transitions.some(
      (t) =>
        t.from === this.currentState &&
        t.event === event &&
        (!t.guard || t.guard(this.context)),
    );
  }

  transition(event: string): boolean {
    const t = this.config.transitions.find(
      (t) =>
        t.from === this.currentState &&
        t.event === event &&
        (!t.guard || t.guard(this.context)),
    );
    if (!t) return false;
    if (t.action) this.context = t.action(this.context);
    this.currentState = t.to;
    return true;
  }
}
