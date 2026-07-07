import os
import re
import subprocess
from pathlib import Path

def get_mapping():
    mapping = {}
    agents = [
        "agent-02-cdawg-opus47.md",
        "agent-03-mrcode-claudecode-opus48.md",
        "agent-04-grokbuild-grok43.md",
        "agent-05-antigravity-claude-sonnet46.md",
        "agent-06-openai-gpt55-xhigh.md",
        "agent-07-antigravity-gemini35-flash.md",
        "agent-08-claude-desktop-opus48.md"
    ]
    for agent in agents:
        try:
            # get old
            old_content = subprocess.check_output(['git', 'show', f'baa38e3:enrollments/{agent}']).decode('utf-8')
            old_match = re.search(r'\b([0-9A-F]{16})\b', old_content)
            if not old_match:
                continue
            old_hex = old_match.group(1).upper()
            
            # get new
            new_content = subprocess.check_output(['git', 'show', f'90ac206:enrollments/{agent}']).decode('utf-8')
            new_match = re.search(r'\b(MCowBQYDK2VwAyEA[A-Za-z0-9+/=]+)\b', new_content)
            if not new_match:
                continue
            new_key = new_match.group(1)
            
            mapping[old_hex] = new_key
            print(f"Mapped {old_hex} -> {new_key[:16]}...")
        except Exception as e:
            print(f"Error processing {agent}: {e}")
            
    # Also add the old Gemini-in-body signature from the papers since we updated that one earlier!
    # Wait, agent-05? In papers/04_ietf_agentic_ai_taxonomy_and_dawn_alignment.md it was:
    # agent_id: E-2A0F1954-1845-001, thumbprint: MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE=
    # We should map MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE= to the new agent-05 or maybe it's a different agent?
    # agent-05 is E-4B7E4B91-1847-001. E-2A0F1954-1845-001 isn't any of these?
    # Wait, earlier I saw in grep:
    # enrollments/agent-05-antigravity-claude-sonnet46.md:83:* **Gemini-in-body** (Antigravity Substrate): unrp_id E-2A0F1954-1845-001, thumbprint MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE=
    # Let's map MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE= to agent-07 (Antigravity Gemini) since it's the Gemini-in-body key?
    # Actually, the user's latest key for Antigravity-Gemini is in agent-07! 
    # Old hex: MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE
        mapping['MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE='] = 'MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE='
    return mapping

def main():
    mapping = get_mapping()
    if not mapping:
        print("No mapping found")
        return
        
    for root, dirs, files in os.walk('.'):
        if '.git' in dirs:
            dirs.remove('.git')
        if 'node_modules' in dirs:
            dirs.remove('node_modules')
            
        for file in files:
            if file.endswith(('.md', '.py', '.txt', '.js', '.json')):
                filepath = os.path.join(root, file)
                try:
                    with open(filepath, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    original_content = content
                    for old_hex, new_key in mapping.items():
                        content = re.sub(r'\b' + old_hex + r'\b', new_key, content)
                    
                    if content != original_content:
                        with open(filepath, 'w', encoding='utf-8') as f:
                            f.write(content)
                        print(f"Updated {filepath}")
                except Exception as e:
                    pass

if __name__ == '__main__':
    main()
