#!/usr/bin/env python3
import json
import os

def fix_timestamps():
    # Define the packet files and their starting timestamps
    files = [
        ("godot/assets/packets/stage1_packets.json",      100_000),
        ("godot/assets/packets/stage1_real_packets.json", 100_000),
        ("godot/assets/packets/stage2_packets.json",      100_000),
        ("godot/assets/packets/stage3_packets.json",      100_000),
        ("godot/assets/packets/stage4_packets.json",      100_000),
        ("godot/assets/packets/stage5_packets.json",      100_000),
    ]
    
    for file_path, start_timestamp in files:
        print(f"Processing {file_path}...")
        
        # Read the JSON file
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Update timestamps incrementally
        current_timestamp = start_timestamp
        for packet in data['packets']:
            packet['timestamp'] = current_timestamp
            current_timestamp += 150_000  # Increment by 150 microseconds
        
        # Write back to file
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        
        print(f"  Updated {len(data['packets'])} packets, timestamps from {start_timestamp} to {current_timestamp - 150}")

if __name__ == "__main__":
    fix_timestamps()
    print("All timestamps fixed!")
