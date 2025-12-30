#!/usr/bin/env python3
"""
Analyze signature failures from stress test output.
Extract failing order debug data and prepare for verification.
"""

import re
import json
from pathlib import Path

def extract_sig_debug_blocks(file_path):
    """Extract all [SIG_DEBUG] blocks from output file."""
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()
    
    # Find all SIG_DEBUG blocks - each order has multiple lines
    blocks = []
    current_block = {}
    order_num = 0
    
    for line in content.split('\n'):
        if '[SIG_DEBUG]' in line:
            # Parse each debug line
            if 'tx_type=' in line:
                # Start of new transaction
                if current_block and 'elements' in current_block:
                    blocks.append(current_block)
                
                order_num += 1
                current_block = {'order_num': order_num}
                
                # Extract tx_type, nonce, expired_at, indices
                m = re.search(r'tx_type=(\d+)', line)
                if m: current_block['tx_type'] = int(m.group(1))
                
                m = re.search(r'nonce=(\d+)', line)
                if m: current_block['nonce'] = int(m.group(1))
                
                m = re.search(r'expired_at=(\d+)', line)
                if m: current_block['expired_at'] = int(m.group(1))
                
                m = re.search(r'account_index=(\d+)', line)
                if m: current_block['account_index'] = int(m.group(1))
                
                m = re.search(r'api_key_index=(\d+)', line)
                if m: current_block['api_key_index'] = int(m.group(1))
            
            elif 'elements=' in line:
                # Extract Goldilocks field elements
                m = re.search(r'elements=\[(.*?)\]', line)
                if m:
                    elements_str = m.group(1)
                    current_block['elements'] = [int(x.strip()) for x in elements_str.split(',')]
            
            elif 'hash_bytes=' in line:
                # Extract hash and public key
                m = re.search(r'hash_bytes=([a-f0-9]+)', line)
                if m: current_block['hash_bytes'] = m.group(1)
                
                m = re.search(r'pubkey=([a-f0-9]+)', line)
                if m: current_block['pubkey'] = m.group(1)
                
                m = re.search(r'sig_hex=([a-f0-9]+)', line)
                if m: current_block['sig_hex'] = m.group(1)
                
                m = re.search(r'sig_b64=([A-Za-z0-9+/=]+)', line)
                if m: current_block['sig_b64'] = m.group(1)
    
    # Add final block if exists
    if current_block and 'elements' in current_block:
        blocks.append(current_block)
    
    return blocks

def find_failing_orders(file_path):
    """Extract failing order numbers from results summary."""
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()
    
    failing_orders = []
    
    # Look for "Order N: code=21120"
    for m in re.finditer(r'Order (\d+): code=21120', content):
        failing_orders.append(int(m.group(1)))
    
    return sorted(set(failing_orders))

def main():
    """Main analysis."""
    output_file = Path('stress_test_output.txt')
    
    print("\n" + "="*70)
    print("SIGNATURE FAILURE ANALYSIS")
    print("="*70 + "\n")
    
    # Get debug blocks
    blocks = extract_sig_debug_blocks(str(output_file))
    print(f"✓ Extracted {len(blocks)} transaction debug blocks\n")
    
    # Get failing orders
    failing = find_failing_orders(str(output_file))
    print(f"✓ Found {len(failing)} signature failures")
    print(f"  Failed order numbers: {failing[:5]}{'...' if len(failing) > 5 else ''}\n")
    
    # Analyze first few failures in detail
    print("-"*70)
    print("DETAILED ANALYSIS OF FIRST 3 FAILING ORDERS")
    print("-"*70 + "\n")
    
    analyzed = 0
    for order_num in failing:
        if analyzed >= 3:
            break
        
        # Find corresponding block
        matching_block = None
        for block in blocks:
            if block.get('order_num') == order_num:
                matching_block = block
                break
        
        if not matching_block:
            print(f"⚠️  Order {order_num}: Debug block not found in output")
            continue
        
        print(f"\nOrder #{order_num}:")
        print(f"  Nonce:          {matching_block.get('nonce', 'N/A')}")
        print(f"  Expired At:     {matching_block.get('expired_at', 'N/A')}")
        print(f"  Account Index:  {matching_block.get('account_index', 'N/A')}")
        print(f"  API Key Index:  {matching_block.get('api_key_index', 'N/A')}")
        
        elements = matching_block.get('elements')
        if elements:
            print(f"  Poseidon Elements (16 fields):")
            print(f"    {elements}")
            print(f"    [chain_id={elements[0]}, tx_type={elements[1]}, nonce={elements[2]},")
            print(f"     expired_at={elements[3]}, account={elements[4]}, api_key={elements[5]}, ...")
        
        if matching_block.get('hash_bytes'):
            print(f"  Hash (40 bytes): {matching_block['hash_bytes']}")
        
        if matching_block.get('pubkey'):
            print(f"  Pubkey (40B):    {matching_block['pubkey']}")
        
        if matching_block.get('sig_hex'):
            print(f"  Sig (hex, 128B): {matching_block['sig_hex'][:48]}...")
            print(f"  Sig (b64):       {matching_block['sig_b64'][:48]}...")
        
        # Save detailed data for verification
        with open(f'failing_order_{order_num}.json', 'w') as f:
            json.dump(matching_block, f, indent=2)
            print(f"  ✓ Saved to failing_order_{order_num}.json")
        
        analyzed += 1
    
    print("\n" + "="*70)
    print("NEXT STEPS:")
    print("="*70)
    print("1. Run: cargo run --release --package goldilocks-crypto --example verify_captured_sig")
    print("   (This will test one of the failing signatures locally)")
    print("\n2. Check if signatures verify locally but fail on server")
    print("   (This indicates a field mismatch or account/key issue)")
    print("\n3. Compare server expectations with our implementation:")
    print("   - Field ordering in Poseidon hash input")
    print("   - Byte canonicalization")
    print("   - Hash output format")
    print("="*70 + "\n")

if __name__ == '__main__':
    main()
