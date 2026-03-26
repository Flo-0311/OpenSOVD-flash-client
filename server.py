from fastapi import FastAPI
from typing import Optional

app = FastAPI()

flash_progress = {}

# ------------------------------------------------------------------
# Health
# ------------------------------------------------------------------
@app.get("/sovd/v1/health")
def health():
    return {"status": "ok"}

# ------------------------------------------------------------------
# Capabilities
# ------------------------------------------------------------------
@app.get("/sovd/v1/capabilities")
def capabilities():
    return {
        "capabilities": [
            {
                "id": "flash_sw",
                "category": "flashing",
                "name": "Flash Software",
                "description": "Flash ECU software",
                "href": "/sovd/v1/components/{id}/flash",
                "methods": ["POST"],
                "parameters": []
            },
            {
                "id": "diag_read",
                "category": "diagnostics",
                "name": "Read DID",
                "description": None,
                "href": "/sovd/v1/components/{id}/data/{did}",
                "methods": ["GET"],
                "parameters": []
            }
        ],
        "server_version": "1.0.0",
        "sovd_version": "1.0"
    }

# ------------------------------------------------------------------
# Components
# ------------------------------------------------------------------
@app.get("/sovd/v1/components")
def components():
    return {
        "components": [
            {
                "id": "ecu_01",
                "name": "Engine ECU",
                "category": "powertrain",
                "href": "/sovd/v1/components/ecu_01",
                "component_type": "native_sovd",
                "status": "available",
                "software_version": "2.1.0",
                "hardware_version": "HW-A",
                "capabilities": ["flash", "diag"],
                "adapter_info": None
            },
            {
                "id": "ecu_02",
                "name": "Body ECU",
                "category": "body",
                "href": "/sovd/v1/components/ecu_02",
                "component_type": "classic_uds",
                "status": "available",
                "software_version": "1.0.0",
                "hardware_version": None,
                "capabilities": ["diag"],
                "adapter_info": None
            }
        ]
    }

# ------------------------------------------------------------------
# Single Component
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}")
def get_component(comp_id: str):
    return {
        "id": comp_id,
        "name": "Engine ECU",
        "category": "powertrain",
        "href": f"/sovd/v1/components/{comp_id}",
        "component_type": "native_sovd",
        "status": "available",
        "software_version": "2.1.0",
        "hardware_version": "HW-A",
        "capabilities": ["flash", "diag"],
        "adapter_info": None
    }

# ------------------------------------------------------------------
# DID Read
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}/data/{did}")
def read_data(comp_id: str, did: str):
    return {
        "id": did,
        "name": "Software Version",
        "value": "2.1.0",
        "unit": None,
        "timestamp": None
    }

# ------------------------------------------------------------------
# DTCs
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}/dtcs")
def dtcs(comp_id: str):
    return [
        {
            "id": "dtc_001",
            "code": "P0300",
            "description": "Misfire",
            "status": "active",
            "severity": "warning",
            "component_id": comp_id
        }
    ]

# ------------------------------------------------------------------
# Monitoring
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}/monitoring")
def monitoring(comp_id: str):
    return {
        "rpm": 3500,
        "temp_c": 90
    }

# ------------------------------------------------------------------
# Logs
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}/logs")
def logs(comp_id: str):
    return [
        {
            "timestamp": "2025-01-01T00:00:00Z",
            "level": "info",
            "message": "Boot OK"
        }
    ]

# ------------------------------------------------------------------
# Flash Start
# ------------------------------------------------------------------
@app.post("/sovd/v1/components/{comp_id}/flash")
def flash(comp_id: str):
    job_id = "job-123"
    flash_progress[job_id] = 0
    return {
        "job_id": job_id,
        "state": "pending"
    }

# ------------------------------------------------------------------
# Flash Status
# ------------------------------------------------------------------
@app.get("/sovd/v1/components/{comp_id}/flash/{job_id}")
def flash_status(comp_id: str, job_id: str):
    progress = flash_progress.get(job_id, 0)

    if progress < 100:
        progress += 25
        flash_progress[job_id] = progress

    state = "running"
    if progress >= 100:
        state = "completed"

    return {
        "state": state,
        "progress": progress
    }