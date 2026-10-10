"""Check the running example over HTTP: python service/tools/smoke.py [endpoint] from the repository root."""
import http.client
import json
import sys
import uuid
from urllib.parse import urlencode, urlsplit

endpoint = urlsplit(sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8080")


def request(method, path, body=None, version=None):
    connection_type = http.client.HTTPSConnection if endpoint.scheme == "https" else http.client.HTTPConnection
    connection = connection_type(endpoint.hostname, endpoint.port, timeout=10)
    try:
        connection.request(method, endpoint.path.rstrip("/") + path,
                           body if isinstance(body, bytes) else (None if body is None else json.dumps(body)),
                           {"Content-Type": "application/json",
                            **({"Crucible-Version": version} if version else {})})
        response = connection.getresponse()
        content = response.read()
        return response.status, json.loads(content) if content else None, dict(response.getheaders())
    finally:
        connection.close()


def error(method, path, body, status, name):
    actual, response, headers = request(method, path, body)
    assert actual == status and response["__type"] == name, (actual, response)
    assert {key.lower(): value for key, value in headers.items()}["x-crucible-error"] == name
    assert {key.lower(): value for key, value in headers.items()}["x-request-id"]
    return response


def main():
    created = []
    token = str(uuid.uuid4())
    first = {"name": "Smoke first", "notes": "Original notes", "clientToken": token}
    try:
        status, response, _ = request("POST", "/items", first)
        assert status == 201, response
        item = response["item"]
        created.append(item["itemId"])
        path = "/items/" + item["itemId"]
        assert request("POST", "/items", first)[:2] == (201, {"item": item})
        conflict = error("POST", "/items", {**first, "name": "Changed intent"}, 409, "ConflictException")
        assert conflict["resourceId"] == item["itemId"] and conflict["resourceType"] == "Item"
        assert request("GET", path)[:2] == (200, {"item": item})
        status, response, _ = request("PUT", path, {"name": "Updated name"})
        assert status == 200 and response["item"]["name"] == "Updated name"
        assert "notes" not in response["item"]
        assert request("GET", path)[1] == response
        for i in range(2):
            status, response, _ = request("POST", "/items", {"name": f"Smoke page {i}", "clientToken": str(uuid.uuid4())})
            assert status == 201, response
            created.append(response["item"]["itemId"])
        seen = []
        query = {"maxResults": 1}
        tokens = set()
        while True:
            status, page, _ = request("GET", "/items?" + urlencode(query))
            assert status == 200 and len(page["items"]) <= query["maxResults"], page
            seen.extend(value["itemId"] for value in page["items"])
            if "nextToken" not in page:
                break
            assert page["nextToken"] not in tokens, "pagination did not advance"
            tokens.add(page["nextToken"])
            query = {"maxResults": 2, "nextToken": page["nextToken"]}
        assert set(created).issubset(seen) and seen == sorted(set(seen)), seen
        for body in [{"name": ""}, {"name": " "}, {"name": "x" * 101}, {"name": "valid"},
                     {"name": "valid", "clientToken": "contains space"}, {"name": "valid", "notes": "x" * 1001, "clientToken": str(uuid.uuid4())}]:
            validation = error("POST", "/items", body, 400, "ValidationException")
            assert validation["reason"] == "fieldValidationFailed" and validation["fieldList"]
        assert error("POST", "/items", b"{", 400, "ValidationException")["reason"] == "cannotParse"
        error("GET", "/items?maxResults=0", None, 400, "ValidationException")
        # Release 1 (migrations.cru): `description`, a default page size, a sunset.
        status, response, headers = request("POST", "/items", {"name": "Release 1", "description": "old name",
                                                                "clientToken": str(uuid.uuid4())}, version="1")
        assert status == 201 and response["item"]["description"] == "old name", response
        created.append(response["item"]["itemId"])
        headers = {key.lower(): value for key, value in headers.items()}
        assert headers["crucible-version"] == "1" and headers["sunset"] == "Fri, 31 Dec 2027 00:00:00 GMT", headers
        status, response, _ = request("GET", "/items/" + created[-1], version="1")
        assert status == 200 and response["item"]["description"] == "old name", response
        assert "notes" not in response["item"], response
        # Release 1 may omit maxResults (the migration fills in its default).
        status, page, _ = request("GET", "/items", version="1")
        assert status == 200 and len(page["items"]) <= 20, page
        error("GET", "/items", None, 400, "ValidationException")
        assert request("GET", "/items", version="9")[0] == 400
        error("GET", "/items?maxResults=101", None, 400, "ValidationException")
        error("GET", "/items?nextToken=forged", None, 400, "ValidationException")
        assert request("DELETE", path)[:2] == (204, None)
        assert request("DELETE", path)[:2] == (204, None)
        not_found = error("GET", path, None, 404, "ResourceNotFoundException")
        assert not_found["resourceId"] == item["itemId"] and not_found["resourceType"] == "Item"
        error("PUT", path, {"name": "Absent"}, 404, "ResourceNotFoundException")
        assert request("POST", "/items", first)[:2] == (201, {"item": item})
        error("GET", path, None, 404, "ResourceNotFoundException")
        error("GET", "/missing", None, 404, "UnknownOperationException")
        print("CRUD, shared state, idempotent retries, pagination, validation and modeled errors passed")
    finally:
        for item_id in created:
            request("DELETE", "/items/" + item_id)


if __name__ == "__main__":
    main()
