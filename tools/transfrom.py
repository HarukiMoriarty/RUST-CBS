import yaml

def load_yaml(file_path):
    with open(file_path, 'r') as f:
        return yaml.safe_load(f)

def save_yaml(data, file_path):
    with open(file_path, 'w') as f:
        f.write("agents:\n")
        for idx, agent in enumerate(data):
            f.write(f"- name: agent{idx}\n")
            f.write(f"  potentialGoals:\n")
            f.write(f"  - [{', '.join(map(str, agent['goal']))}]\n")
            f.write(f"  start: [{', '.join(map(str, agent['start']))}]\n")

def main():
    input_path = 'debug.yaml'
    output_path = 'transform.yaml'

    original_data = load_yaml(input_path)
    save_yaml(original_data, output_path)

if __name__ == "__main__":
    main()
