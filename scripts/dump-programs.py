import json
import os
import subprocess
import sys
from pathlib import Path

def ensure_binaries_dir():
    binaries_dir = Path("./binaries")
    binaries_dir.mkdir(exist_ok=True)
    return binaries_dir

def dump_program(program_id, name, binaries_dir):
    output_path = binaries_dir / name
    
    print(f"Dumping program {name} ({program_id})...")
    try:
        result = subprocess.run(
            ["solana", "program", "dump", program_id, str(output_path)],
            capture_output=True,
            text=True,
            check=True
        )
        print(f"Successfully dumped {name} to {output_path}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error dumping program {name}: {e.stderr}")
        return False

def main():
    script_dir = Path(__file__).parent
    json_path = script_dir / "programs.json"
    
    if not json_path.exists():
        print(f"Error: {json_path} not found")
        print("Please create a programs.json file with format:")
        print('''[
    {
        "program_id": "your_program_id",
        "name": "program_name"
    },
    ...
]''')
        sys.exit(1)
    
    binaries_dir = ensure_binaries_dir()
    try:
        with open(json_path, 'r') as f:
            programs = json.load(f)
            
        if not isinstance(programs, list):
            raise ValueError("JSON file must contain a list of programs")
            
        for program in programs:
            if not isinstance(program, dict):
                raise ValueError("Each program must be an object")
            if "program_id" not in program or "name" not in program:
                raise ValueError("Each program must have 'program_id' and 'name' fields")
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON file: {e}")
        sys.exit(1)
    except ValueError as e:
        print(f"Invalid JSON format: {e}")
        sys.exit(1)
    
    success_count = 0
    for program in programs:
        if dump_program(program["program_id"], program["name"], binaries_dir):
            success_count += 1

    print(f"\nDump complete!")
    print(f"Successfully dumped {success_count} out of {len(programs)} programs")
    if success_count < len(programs):
        print(f"Failed to dump {len(programs) - success_count} programs")

if __name__ == "__main__":
    main()