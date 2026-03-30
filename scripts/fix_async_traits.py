import re
import sys

def transform_file(path):
    with open(path, 'r') as f:
        content = f.read()

    # Pattern 1: fn name(...) -> impl Future<Output = Result<T, E>> + Send { ... async move { body } }
    # This regex is a bit complex to handle multiline and nested braces.
    # We'll try a simpler approach first: replace the signature and then the async move block.
    
    # 1. Replace signatures: fn name(...) -> impl Future<Output = X> + Send
    content = re.sub(
        r'fn\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)\s*->\s*impl\s+Future<Output\s*=\s*([^>]+)>\s*\+\s*Send\s*\{',
        r'async fn \1(\2) -> \3 {',
        content
    )

    # 2. Remove 'async move {' and its closing '}' at the end of function bodies.
    # This is tricky because of nested braces.
    # For MockStorage, most bodies are:
    # {
    #    let ...;
    #    async move { ... }
    # }
    
    # We'll look for 'async move {' and remove it, then find the corresponding '}' and remove it.
    
    lines = content.split('\n')
    new_lines = []
    in_async_move = False
    brace_count = 0
    
    for line in lines:
        if 'async move {' in line:
            # Replace 'async move {' with just its indentation if there's other code on the line,
            # but usually it's on its own line or at the end.
            line = line.replace('async move {', '')
            in_async_move = True
            brace_count = 1
            if not line.strip():
                continue
        
        if in_async_move:
            brace_count += line.count('{')
            brace_count -= line.count('}')
            
            if brace_count <= 0:
                # This was the closing brace of async move { ... }
                # We need to remove the last '}'
                last_brace_idx = line.rfind('}')
                if last_brace_idx != -1:
                    line = line[:last_brace_idx] + line[last_brace_idx+1:]
                in_async_move = False
                brace_count = 0
        
        new_lines.append(line)
    
    transformed = '\n'.join(new_lines)
    
    # Remove leftover empty lines if any
    # transformed = re.sub(r'\n\s*\n\s*\n', r'\n\n', transformed)

    with open(path, 'w') as f:
        f.write(transformed)

if __name__ == "__main__":
    transform_file(sys.argv[1])
