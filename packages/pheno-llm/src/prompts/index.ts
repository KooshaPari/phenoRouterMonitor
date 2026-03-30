// Prompt Management and Template System
// NEW module — Prompt templating, composition, and registry

// ============================================================================
// CORE TYPES
// ============================================================================

/**
 * Prompt variable binding
 */
export interface PromptVariable {
  name: string;
  description?: string;
  default?: string;
  required?: boolean;
  type?: 'string' | 'number' | 'boolean' | 'json';
}

/**
 * Prompt template
 */
export interface PromptTemplate {
  name: string;
  version: string;
  description?: string;
  template: string;
  variables: PromptVariable[];
  examples?: {
    input: Record<string, unknown>;
    output: string;
  }[];
}

/**
 * Compiled prompt ready to send to LLM
 */
export interface CompiledPrompt {
  systemPrompt?: string;
  userPrompt: string;
  metadata: {
    templateName: string;
    compiledAt: Date;
    variablesUsed: Record<string, unknown>;
  };
}

/**
 * Prompt builder for fluent construction
 */
export interface PromptBuilder {
  /**
   * Set system prompt
   */
  system(prompt: string): this;

  /**
   * Set user prompt
   */
  user(prompt: string): this;

  /**
   * Add a section
   */
  section(title: string, content: string): this;

  /**
   * Add context
   */
  context(key: string, value: string | Record<string, unknown>): this;

  /**
   * Add a list
   */
  list(items: string[]): this;

  /**
   * Add JSON
   */
  json(obj: Record<string, unknown>): this;

  /**
   * Build the prompt
   */
  build(): CompiledPrompt;
}

/**
 * Prompt registry
 */
export interface PromptRegistry {
  /**
   * Register a template
   */
  register(template: PromptTemplate): void;

  /**
   * Get a template by name
   */
  get(name: string): PromptTemplate | null;

  /**
   * List all templates
   */
  list(): PromptTemplate[];

  /**
   * Remove a template
   */
  unregister(name: string): void;
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

/**
 * Simple prompt template implementation with variable interpolation
 */
export class SimplePromptTemplate implements PromptTemplate {
  name: string;
  version: string;
  description?: string;
  template: string;
  variables: PromptVariable[];
  examples?: Array<{ input: Record<string, unknown>; output: string }>;

  constructor(config: PromptTemplate) {
    this.name = config.name;
    this.version = config.version;
    this.description = config.description;
    this.template = config.template;
    this.variables = config.variables;
    this.examples = config.examples;
  }

  /**
   * Compile template with variables
   */
  compile(variables: Record<string, unknown>): string {
    let result = this.template;

    // Replace variables in template
    for (const [key, value] of Object.entries(variables)) {
      const placeholder = `{{${key}}}`;
      result = result.replaceAll(placeholder, String(value));
    }

    // Replace remaining variables with defaults
    for (const variable of this.variables) {
      const placeholder = `{{${variable.name}}}`;
      if (result.includes(placeholder)) {
        if (variable.default) {
          result = result.replaceAll(placeholder, variable.default);
        } else if (variable.required) {
          throw new Error(`Required variable missing: ${variable.name}`);
        } else {
          result = result.replaceAll(placeholder, '');
        }
      }
    }

    return result;
  }
}

/**
 * Fluent prompt builder
 */
export class FluentPromptBuilder implements PromptBuilder {
  private systemPrompt?: string;
  private userPrompt = '';
  private sections: Array<{ title: string; content: string }> = [];
  private contextItems: Record<string, string> = {};

  system(prompt: string): this {
    this.systemPrompt = prompt;
    return this;
  }

  user(prompt: string): this {
    this.userPrompt = prompt;
    return this;
  }

  section(title: string, content: string): this {
    this.sections.push({ title, content });
    return this;
  }

  context(key: string, value: string | Record<string, unknown>): this {
    this.contextItems[key] = typeof value === 'string' ? value : JSON.stringify(value);
    return this;
  }

  list(items: string[]): this {
    this.userPrompt += '\n' + items.map((item) => `- ${item}`).join('\n');
    return this;
  }

  json(obj: Record<string, unknown>): this {
    this.userPrompt += '\n```json\n' + JSON.stringify(obj, null, 2) + '\n```';
    return this;
  }

  build(): CompiledPrompt {
    let finalUserPrompt = this.userPrompt;

    // Add sections
    if (this.sections.length > 0) {
      finalUserPrompt = this.sections
        .map((s) => `## ${s.title}\n\n${s.content}`)
        .join('\n\n') + (finalUserPrompt ? '\n\n' + finalUserPrompt : '');
    }

    // Add context
    if (Object.keys(this.contextItems).length > 0) {
      const contextStr = Object.entries(this.contextItems)
        .map(([key, value]) => `- ${key}: ${value}`)
        .join('\n');
      finalUserPrompt = `**Context:**\n${contextStr}\n\n${finalUserPrompt}`;
    }

    return {
      systemPrompt: this.systemPrompt,
      userPrompt: finalUserPrompt,
      metadata: {
        templateName: 'custom',
        compiledAt: new Date(),
        variablesUsed: {},
      },
    };
  }
}

/**
 * In-memory prompt registry
 */
export class InMemoryPromptRegistry implements PromptRegistry {
  private templates: Map<string, PromptTemplate> = new Map();

  register(template: PromptTemplate): void {
    this.templates.set(template.name, template);
  }

  get(name: string): PromptTemplate | null {
    return this.templates.get(name) ?? null;
  }

  list(): PromptTemplate[] {
    return Array.from(this.templates.values());
  }

  unregister(name: string): void {
    this.templates.delete(name);
  }
}

// ============================================================================
// COMMON PROMPT TEMPLATES
// ============================================================================

/**
 * System prompt for code generation
 */
export const CODE_GENERATION_SYSTEM = `You are an expert software engineer.
Your task is to generate high-quality, production-ready code.
Follow best practices, include proper error handling, and write clear documentation.
Prefer using established libraries over hand-rolling solutions.`;

/**
 * System prompt for analysis
 */
export const ANALYSIS_SYSTEM = `You are a thorough analyst.
Your task is to analyze the provided content carefully and provide detailed insights.
Be objective, cite evidence, and explain your reasoning clearly.`;

/**
 * System prompt for creative writing
 */
export const CREATIVE_SYSTEM = `You are a creative writer with excellent storytelling skills.
Your task is to create engaging, original content that captures the reader's attention.
Use vivid language and compelling narratives.`;

// ============================================================================
// TEMPLATE FACTORY
// ============================================================================

/**
 * Create a code generation template
 */
export function createCodeTemplate(
  name: string,
  description: string,
  template: string,
): PromptTemplate {
  return {
    name,
    version: '1.0.0',
    description,
    template,
    variables: [
      { name: 'language', description: 'Programming language', required: true },
      { name: 'requirements', description: 'Code requirements', required: true },
      { name: 'style', description: 'Code style preferences', required: false },
    ],
  };
}

/**
 * Create an analysis template
 */
export function createAnalysisTemplate(
  name: string,
  description: string,
  template: string,
): PromptTemplate {
  return {
    name,
    version: '1.0.0',
    description,
    template,
    variables: [
      { name: 'subject', description: 'Subject to analyze', required: true },
      { name: 'context', description: 'Additional context', required: false },
      { name: 'focus', description: 'Specific focus areas', required: false },
    ],
  };
}
