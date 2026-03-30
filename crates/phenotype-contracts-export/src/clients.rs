//! Polyglot client code generation (TypeScript and Go POCs).

/// Generates TypeScript client code.
pub struct TypeScriptClientGenerator;

impl TypeScriptClientGenerator {
    /// Generates a complete TypeScript client implementation.
    pub fn generate() -> String {
        let mut code = String::new();

        code.push_str("// Auto-generated Phenotype TypeScript Client\n");
        code.push_str("// Generated from Phenotype Contracts v1.0.0\n\n");

        code.push_str("import axios, { AxiosInstance } from 'axios';\n\n");

        // Interfaces
        code.push_str("// Inbound Port Interfaces\n\n");
        code.push_str("export interface IUseCase {\n");
        code.push_str("  execute(request: any): Promise<any>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface ICommandHandler {\n");
        code.push_str("  handle(command: any): Promise<void>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface IQueryHandler {\n");
        code.push_str("  handle(query: any): Promise<any>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface IEventHandler {\n");
        code.push_str("  handle(event: any): Promise<void>;\n");
        code.push_str("}\n\n");

        code.push_str("// Outbound Port Interfaces\n\n");
        code.push_str("export interface IRepository {\n");
        code.push_str("  save(id: string, entity: any): Promise<void>;\n");
        code.push_str("  get(id: string): Promise<any>;\n");
        code.push_str("  delete(id: string): Promise<void>;\n");
        code.push_str("  list(): Promise<any[]>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface ICachePort {\n");
        code.push_str("  get(key: string): Promise<any | null>;\n");
        code.push_str("  set(key: string, value: any): Promise<void>;\n");
        code.push_str("  invalidate(key: string): Promise<void>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface IEventBus {\n");
        code.push_str("  publish(event: any): Promise<void>;\n");
        code.push_str("  publishBatch(events: any[]): Promise<void>;\n");
        code.push_str("}\n\n");

        code.push_str("export interface ISecretManager {\n");
        code.push_str("  get(name: string): Promise<string>;\n");
        code.push_str("  set(name: string, value: string): Promise<void>;\n");
        code.push_str("  delete(name: string): Promise<void>;\n");
        code.push_str("}\n\n");

        code.push_str("// Client Implementation\n\n");
        code.push_str("export class PhenotypeClient {\n");
        code.push_str("  private client: AxiosInstance;\n\n");

        code.push_str("  constructor(baseURL: string = 'http://localhost:8080') {\n");
        code.push_str("    this.client = axios.create({\n");
        code.push_str("      baseURL,\n");
        code.push_str("      headers: {\n");
        code.push_str("        'Content-Type': 'application/json',\n");
        code.push_str("      },\n");
        code.push_str("    });\n");
        code.push_str("  }\n\n");

        code.push_str("  // UseCase operations\n");
        code.push_str("  async executeUseCase(request: any): Promise<any> {\n");
        code.push_str("    return this.client.post('/api/use-case/execute', request);\n");
        code.push_str("  }\n\n");

        code.push_str("  // Command operations\n");
        code.push_str("  async handleCommand(command: any): Promise<void> {\n");
        code.push_str("    await this.client.post('/api/commands', command);\n");
        code.push_str("  }\n\n");

        code.push_str("  // Query operations\n");
        code.push_str("  async handleQuery(query: any): Promise<any> {\n");
        code.push_str("    return this.client.post('/api/queries', query);\n");
        code.push_str("  }\n\n");

        code.push_str("  // Event operations\n");
        code.push_str("  async publishEvent(event: any): Promise<void> {\n");
        code.push_str("    await this.client.post('/api/events', event);\n");
        code.push_str("  }\n\n");

        code.push_str("  // Health check\n");
        code.push_str("  async health(): Promise<any> {\n");
        code.push_str("    return this.client.get('/health');\n");
        code.push_str("  }\n");
        code.push_str("}\n\n");

        code.push_str("export default PhenotypeClient;\n");

        code
    }
}

/// Generates Go client code.
pub struct GoClientGenerator;

impl GoClientGenerator {
    /// Generates a complete Go client implementation.
    pub fn generate() -> String {
        let mut code = String::new();

        code.push_str("// Auto-generated Phenotype Go Client\n");
        code.push_str("// Generated from Phenotype Contracts v1.0.0\n\n");

        code.push_str("package phenotype\n\n");

        code.push_str("import (\n");
        code.push_str("  \"context\"\n");
        code.push_str("  \"encoding/json\"\n");
        code.push_str("  \"net/http\"\n");
        code.push_str("  \"github.com/go-resty/resty/v2\"\n");
        code.push_str(")\n\n");

        // Interfaces
        code.push_str("// Inbound Ports\n\n");
        code.push_str("type UseCase interface {\n");
        code.push_str("  Execute(ctx context.Context, request interface{}) (interface{}, error)\n");
        code.push_str("}\n\n");

        code.push_str("type CommandHandler interface {\n");
        code.push_str("  Handle(ctx context.Context, command interface{}) error\n");
        code.push_str("}\n\n");

        code.push_str("type QueryHandler interface {\n");
        code.push_str("  Handle(ctx context.Context, query interface{}) (interface{}, error)\n");
        code.push_str("}\n\n");

        code.push_str("type EventHandler interface {\n");
        code.push_str("  Handle(ctx context.Context, event interface{}) error\n");
        code.push_str("}\n\n");

        // Outbound Ports
        code.push_str("// Outbound Ports\n\n");
        code.push_str("type Repository interface {\n");
        code.push_str("  Save(ctx context.Context, id string, entity interface{}) error\n");
        code.push_str("  Get(ctx context.Context, id string) (interface{}, error)\n");
        code.push_str("  Delete(ctx context.Context, id string) error\n");
        code.push_str("  List(ctx context.Context) ([]interface{}, error)\n");
        code.push_str("}\n\n");

        code.push_str("type CachePort interface {\n");
        code.push_str("  Get(ctx context.Context, key string) (interface{}, error)\n");
        code.push_str("  Set(ctx context.Context, key string, value interface{}) error\n");
        code.push_str("  Invalidate(ctx context.Context, key string) error\n");
        code.push_str("}\n\n");

        code.push_str("type EventBus interface {\n");
        code.push_str("  Publish(ctx context.Context, event interface{}) error\n");
        code.push_str("  PublishBatch(ctx context.Context, events []interface{}) error\n");
        code.push_str("}\n\n");

        code.push_str("type SecretManager interface {\n");
        code.push_str("  Get(ctx context.Context, name string) (string, error)\n");
        code.push_str("  Set(ctx context.Context, name, value string) error\n");
        code.push_str("  Delete(ctx context.Context, name string) error\n");
        code.push_str("}\n\n");

        // Client
        code.push_str("// PhenotypeClient is the HTTP client for interacting with Phenotype services\n");
        code.push_str("type PhenotypeClient struct {\n");
        code.push_str("  client *resty.Client\n");
        code.push_str("  baseURL string\n");
        code.push_str("}\n\n");

        code.push_str("// NewPhenotypeClient creates a new Phenotype client\n");
        code.push_str("func NewPhenotypeClient(baseURL string) *PhenotypeClient {\n");
        code.push_str("  return &PhenotypeClient{\n");
        code.push_str("    client: resty.New(),\n");
        code.push_str("    baseURL: baseURL,\n");
        code.push_str("  }\n");
        code.push_str("}\n\n");

        code.push_str("// ExecuteUseCase executes a use case\n");
        code.push_str("func (c *PhenotypeClient) ExecuteUseCase(ctx context.Context, request interface{}) (interface{}, error) {\n");
        code.push_str("  var result interface{}\n");
        code.push_str("  resp, err := c.client.R().\n");
        code.push_str("    SetContext(ctx).\n");
        code.push_str("    SetBody(request).\n");
        code.push_str("    SetResult(&result).\n");
        code.push_str("    Post(c.baseURL + \"/api/use-case/execute\")\n");
        code.push_str("  if err != nil {\n");
        code.push_str("    return nil, err\n");
        code.push_str("  }\n");
        code.push_str("  if resp.StatusCode() != http.StatusOK {\n");
        code.push_str("    return nil, ErrUnexpectedStatus(resp.StatusCode())\n");
        code.push_str("  }\n");
        code.push_str("  return result, nil\n");
        code.push_str("}\n\n");

        code.push_str("// Health checks the service health\n");
        code.push_str("func (c *PhenotypeClient) Health(ctx context.Context) error {\n");
        code.push_str("  resp, err := c.client.R().\n");
        code.push_str("    SetContext(ctx).\n");
        code.push_str("    Get(c.baseURL + \"/health\")\n");
        code.push_str("  if err != nil {\n");
        code.push_str("    return err\n");
        code.push_str("  }\n");
        code.push_str("  if resp.StatusCode() != http.StatusOK {\n");
        code.push_str("    return ErrUnexpectedStatus(resp.StatusCode())\n");
        code.push_str("  }\n");
        code.push_str("  return nil\n");
        code.push_str("}\n\n");

        code.push_str("// ErrUnexpectedStatus returns an error for unexpected HTTP status\n");
        code.push_str("func ErrUnexpectedStatus(status int) error {\n");
        code.push_str("  return json.NewEncoder(nil).Encode(map[string]interface{}{\n");
        code.push_str("    \"error\": \"unexpected status\",\n");
        code.push_str("    \"status\": status,\n");
        code.push_str("  })\n");
        code.push_str("}\n");

        code
    }
}
