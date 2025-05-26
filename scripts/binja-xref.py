import json
import hashlib
from binaryninja import *

def get_function_signature(bv, func):
    """Extract the raw hex of first 6 instructions in a function."""
    if func.start < 0x100000120 or func.start >= 0x200000000:
        return None
        
    instructions = []
    instruction_bytes = bytearray()
    
    print(f"Analyzing function: {func.name} at {hex(func.start)}")
    
    for block in func.basic_blocks:
        for insn in block.disassembly_text:
            if insn.tokens and len(instructions) < 6:
                addr = insn.address
                if len(block.disassembly_text) > block.disassembly_text.index(insn) + 1:
                    next_insn = block.disassembly_text[block.disassembly_text.index(insn) + 1]
                    length = next_insn.address - addr
                else:
                    length = 4
                
                insn_bytes = bv.read(addr, length)
                instruction_bytes.extend(insn_bytes)
                
                instructions.append({
                    "address": addr,
                    "text": ''.join(token.text for token in insn.tokens),
                    "bytes": insn_bytes.hex()
                })
                
                print(f"  Instruction {len(instructions)}: {hex(addr)} - {''.join(token.text for token in insn.tokens)} - {insn_bytes.hex()}")
            
            if len(instructions) >= 6:
                break
        if len(instructions) >= 6:
            break
    
    # If we don't have enough instructions, return None
    if len(instructions) < 6:
        print(f"  Skipping {func.name} - not enough instructions ({len(instructions)} found)")
        return None
    
    hex_signature = instruction_bytes.hex()
    sha1_hash = hashlib.sha1(instruction_bytes).hexdigest()
    
    print(f"  Signature for {func.name}: {hex_signature[:60]}... ({len(hex_signature)} chars)")
    print(f"  Hash: {sha1_hash}")
    
    return {
        "raw_bytes": instruction_bytes,
        "hex": hex_signature,
        "hash": sha1_hash,
        "start_addr": func.start,
        "name": func.name
    }

def process_binary_file(base_signatures, binary_path, binary_type, existing_hashes):
    """Process a binary file and find matches with base signatures."""
    print(f"\nProcessing {binary_type} file...")
    matches = set()
    
    try:
        with open(binary_path, 'rb') as f:
            binary_data = f.read()
            for sig in base_signatures:
                if (sig['hash'], binary_type) in existing_hashes:
                    continue
                    
                raw_bytes = sig['raw_bytes']
                if raw_bytes in binary_data:
                    print(f"Found match in {binary_type} for function {sig['name']} at {hex(sig['start_addr'])}")
                    matches.add(sig['hash'])
    except Exception as e:
        print(f"Error processing {binary_type} file: {str(e)}")
    
    return matches

def update_matches(matches, base_signatures, new_matches, base_type, binary_type):
    """Update matches list with new matches found."""
    for sig in base_signatures:
        if sig['hash'] in new_matches:
            existing_match = next((m for m in matches if m['hash'] == sig['hash']), None)
            if existing_match:
                if binary_type not in existing_match['found_in']:
                    existing_match['found_in'].append(binary_type)
            else:
                matches.append({
                    "name": sig['name'],
                    "found_in": [base_type, binary_type],
                    "hex": sig['hex'],
                    "hash": sig['hash'],
                    "address": hex(sig['start_addr'])
                })
    return matches

def deduplicate_matches(matches):
    """
    Deduplicate matches based on hash and clean up found_in lists.
    Returns a new list with unique entries.
    """
    deduped = {}
    
    for match in matches:
        if match['hash'] in deduped:
            existing = deduped[match['hash']]
            existing['found_in'] = list(set(existing['found_in'] + match['found_in']))
        else:
            match['found_in'] = list(set(match['found_in']))
            deduped[match['hash']] = match
    
    return sorted(deduped.values(), key=lambda x: x['name'])

def main():
    base_bv = current_view
    
    if base_bv is None:
        print("No binary view is currently active. Please open a file in Binary Ninja first.")
        return
    
    print(f"Using current view as Base: {base_bv.file.filename}")
    print("\n=== Base File Configuration ===")
    print("Available types: Native, Anchor, Pinocchio")
    base_type = get_text_line_input("Base Type", "Enter the type of the base file (default: Native):").decode().strip()
    if not base_type:
        base_type = "Native"
    elif base_type not in ["Native", "Anchor", "Pinocchio"]:
        print("Invalid base type. Must be 'Native', 'Anchor', or 'Pinocchio'")
        return
    
    print(f"Using {base_type} as base type")
    
    use_existing = get_text_line_input("Use Existing Results", "Load existing results file? (Y/N):").decode().strip().upper()
    
    matches = []
    if use_existing == 'Y':
        input_json_path = get_open_filename_input("Select existing JSON file")
        if input_json_path:
            try:
                with open(input_json_path, 'r') as f:
                    matches = json.load(f)
                print(f"Loaded {len(matches)} existing matches from {input_json_path}")
            except Exception as e:
                print(f"Error loading existing JSON file: {str(e)}")
                return
    
    print(f"\nAnalyzing Base file...")
    base_signatures = []
    for func in base_bv.functions:
        signature = get_function_signature(base_bv, func)
        if signature:
            base_signatures.append(signature)
    
    print(f"Generated signatures for {len(base_signatures)} functions in Base file")
    
    existing_hashes = {(m['hash'], bt) for m in matches for bt in m['found_in'] if bt != base_type}
    
    while True:
        binary_path = get_open_filename_input("Select binary file to analyze")
        if not binary_path:
            print("No file selected. Exiting.")
            break
        
        print("\nAvailable types: Native, Anchor, Pinocchio")
        binary_type = get_text_line_input("Binary Type", "Enter binary type:").decode().strip()
        if binary_type not in ["Native", "Anchor", "Pinocchio"]:
            print("Invalid binary type. Must be 'Native', 'Anchor', or 'Pinocchio'")
            continue
        
        new_matches = process_binary_file(base_signatures, binary_path, binary_type, existing_hashes)
        matches = update_matches(matches, base_signatures, new_matches, base_type, binary_type)
        existing_hashes.update((h, binary_type) for h in new_matches)
        continue_analysis = get_text_line_input("Continue", "Process another binary? (Y/N):").decode().strip().upper()
        if continue_analysis != 'Y':
            break
    
    if matches:
        print("\nDeduplicating matches...")
        original_count = len(matches)
        matches = deduplicate_matches(matches)
        print(f"Deduplicated {original_count} matches to {len(matches)} unique entries")
        
        output_path = get_save_filename_input("Save output JSON file")
        if output_path:
            with open(output_path, 'w') as f:
                json.dump(matches, f, indent=2)
            print(f"Results saved to {output_path}")
            print(f"Total unique matches found: {len(matches)}")

if __name__ == "__main__":
    main()
