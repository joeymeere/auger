import json
from binaryninja import *

def load_signatures(json_path):
    """Load function signatures from a JSON file."""
    try:
        with open(json_path, 'r') as f:
            signatures = json.load(f)
        print(f"Loaded {len(signatures)} signatures from {json_path}")
        return signatures
    except Exception as e:
        print(f"Error loading JSON file: {str(e)}")
        return None

def bytes_from_hex(hex_string):
    """Convert a hex string to bytes."""
    return bytes.fromhex(hex_string)

def find_and_rename_functions(bv, signatures):
    """Find functions matching signatures and rename them."""
    matches = 0
    
    for func in bv.functions:
        if func.start < 0x100000120 or func.start >= 0x200000000:
            continue
            
        func_bytes = bv.read(func.start, 48)  # Read enough bytes to cover typical signature length
        if not func_bytes:
            continue
            
        for sig in signatures:
            sig_bytes = bytes_from_hex(sig['hex'])
            
            if func_bytes.startswith(sig_bytes):
                old_name = func.name
                new_name = sig['name']
                
                if old_name == new_name:
                    continue
                    
                func.name = new_name
                matches += 1
                print(f"Renamed function at {hex(func.start)} from {old_name} to {new_name}")
    
    return matches

def main():
    bv = current_view
    
    if bv is None:
        print("No binary view is currently active. Please open a file in Binary Ninja first.")
        return
    
    print(f"Using current view: {bv.file.filename}")
    
    json_path = get_open_filename_input("Select signatures JSON file")
    if not json_path:
        print("No JSON file selected. Exiting.")
        return
    
    signatures = load_signatures(json_path)
    if not signatures:
        return
    
    matches = find_and_rename_functions(bv, signatures)
    print(f"\nRenamed {matches} functions based on signature matches")

if __name__ == "__main__":
    main() 