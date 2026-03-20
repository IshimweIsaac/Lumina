import requests
import concurrent.futures
import time

URL = "http://127.0.0.1:8080/api/event"

def send_random_event(i):
    bike_id = (i % 1000) + 1
    name = f"moto{bike_id}"
    try:
        r = requests.post(URL, json={
            "instance": name,
            "field": "isBusy",
            "value": i % 2 == 0
        })
        return r.status_code
    except Exception as e:
        return str(e)

print("Starting stress test...")
with concurrent.futures.ThreadPoolExecutor(max_workers=50) as executor:
    results = list(executor.map(send_random_event, range(500)))

print(f"Finished. Statuses: {set(results)}")
