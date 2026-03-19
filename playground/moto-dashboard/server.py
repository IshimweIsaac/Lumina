import os
import sys
from flask import Flask, request, jsonify, send_from_directory
from flask_cors import CORS

# Add lumina_py to sys.path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "crates", "lumina-ffi")))
from lumina_py import LuminaRuntime

app = Flask(__name__, static_folder="public", static_url_path="/")
CORS(app)

SOURCE_HEADER = """
entity Moto {
  battery: Number
  isBusy: Boolean
  isCharging: Boolean
  isLocked: Boolean
  isIdle := not isBusy and not isCharging
  isLowBattery := battery < 20
  isAvailable  := not isBusy and not isCharging and battery > 15 and not isLocked
  status := if isLocked then "locked" else if isCharging then "charging" else if isBusy then "in use" else "idle"
}
"""

def generate_fleet_source(count):
    lines = []
    # Generation rules for each instance
    for i in range(1, count + 1):
        name = f"moto{i}"
        lines.append(f"let {name} = Moto {{ battery: {80 + (i % 20)}, isBusy: false, isCharging: false, isLocked: false }}")
        lines.append(f'rule "{name} low battery" {{ when {name}.isLowBattery becomes true then show "Lumina Alert: {name} Battery Low!" }}')
        lines.append(f'rule "{name} fully charged" {{ when {name}.battery >= 100 becomes true then update {name}.isCharging to false }}')
        lines.append(f'rule "{name} auto lock" {{ when {name}.isIdle becomes true for 10 s then update {name}.isLocked to true }}')
        lines.append(f'rule "{name} unlock" {{ when {name}.isBusy becomes true then update {name}.isLocked to false }}')
        # Guard against conflicting states
        lines.append(f'rule "{name} busy guard" {{ when {name}.isBusy becomes true and {name}.isCharging then update {name}.isCharging to false }}')
        lines.append(f'rule "{name} charge guard" {{ when {name}.isCharging becomes true and {name}.isBusy then update {name}.isBusy to false }}')
    return "\n".join(lines)

SOURCE = SOURCE_HEADER + generate_fleet_source(1000)


# Initialize Lumina runtime
try:
    rt = LuminaRuntime.from_source(SOURCE)
    print("Lumina Runtime initialized successfully.")
except Exception as e:
    print(f"Error initializing Lumina Runtime: {e}")
    rt = None

@app.route("/")
def index():
    return send_from_directory("public", "index.html")

@app.route("/api/state", methods=["GET"])
def get_state():
    if not rt:
        return jsonify({"error": "Runtime not initialized"}), 500
    try:
        rt.tick()
        state = rt.export_state()
        state["messages"] = rt.get_messages()
        return jsonify(state)
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/api/event", methods=["POST"])
def apply_event():
    if not rt:
        return jsonify({"error": "Runtime not initialized"}), 500
    data = request.json
    instance = data.get("instance")
    field = data.get("field")
    value = data.get("value")
    
    if not instance or not field or value is None:
        return jsonify({"error": "Missing instance, field, or value"}), 400
        
    try:
        result = rt.apply_event(instance, field, value)
        return jsonify(result)
    except Exception as e:
        return jsonify({"error": str(e)}), 400

if __name__ == "__main__":
    app.run(debug=True, port=8080)
