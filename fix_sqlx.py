#!/usr/bin/env python3
import re
import sys

def fix_sqlx_query(content):
    # Pattern to match sqlx::query!(...) with parameters
    pattern = r'sqlx::query!\(\s*r#"\s*(.*?)\s*"#,\s*(.*?)\s*\)'
    
    def replace_func(match):
        sql = match.group(1).strip()
        params = match.group(2).strip()
        
        # Split parameters by comma, but be careful with nested calls
        param_list = []
        current_param = ""
        paren_depth = 0
        
        for char in params:
            if char == '(':
                paren_depth += 1
                current_param += char
            elif char == ')':
                paren_depth -= 1
                current_param += char
            elif char == ',' and paren_depth == 0:
                param_list.append(current_param.strip())
                current_param = ""
            else:
                current_param += char
        
        if current_param.strip():
            param_list.append(current_param.strip())
        
        # Build the new query
        result = f'sqlx::query(\n            r#"\n            {sql}\n            "#\n        )'
        
        for param in param_list:
            result += f'\n        .bind({param})'
        
        return result
    
    return re.sub(pattern, replace_func, content, flags=re.DOTALL)

def main():
    if len(sys.argv) != 2:
        print("Usage: python fix_sqlx.py <file>")
        sys.exit(1)
    
    filename = sys.argv[1]
    
    with open(filename, 'r') as f:
        content = f.read()
    
    fixed_content = fix_sqlx_query(content)
    
    with open(filename, 'w') as f:
        f.write(fixed_content)
    
    print(f"Fixed {filename}")

if __name__ == "__main__":
    main()
