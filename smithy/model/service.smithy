$version: "2"
namespace example.service

use crucible.protocols#json
use crucible.errors#ValidationException
use crucible.errors#InternalServerException
use crucible.errors#UnknownOperationException

@json
service MyService {
    version: "1"
    resources: [Item]
    errors: [ValidationException, InternalServerException, UnknownOperationException]
}

resource Item {
    identifiers: { itemId: ItemId }
    create: CreateItem
    read: GetItem
    update: UpdateItem
    delete: DeleteItem
    list: ListItems
}

@length(min: 36, max: 36)
@pattern("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
string ItemId

@length(min: 1, max: 100)
@pattern("^.*\\S.*$")
string ItemName

@length(max: 1000)
string ItemDescription

@length(min: 1, max: 128)
@pattern("^[!-~]+$")
string ClientToken

@length(min: 1, max: 64)
string NextToken

structure ItemDetails {
    @required
    itemId: ItemId
    @required
    name: ItemName
    description: ItemDescription
}

list ItemList { member: ItemDetails }

/// Creates an item; retries with the same token and fields return the original result for eight hours.
@http(method: "POST", uri: "/items", code: 201)
operation CreateItem {
    input: CreateItemInput
    output: CreateItemOutput
    errors: [ConflictException, ServiceQuotaExceededException]
}

@input
structure CreateItemInput {
    @required
    name: ItemName
    description: ItemDescription
    @idempotencyToken
    clientToken: ClientToken
}

@output
structure CreateItemOutput {
    @required
    item: ItemDetails
}

/// Retrieves an item by its immutable identifier.
@readonly
@http(method: "GET", uri: "/items/{itemId}", code: 200)
operation GetItem {
    input: GetItemInput
    output: GetItemOutput
    errors: [ResourceNotFoundException]
}

@input
structure GetItemInput {
    @required
    @httpLabel
    itemId: ItemId
}

@output
structure GetItemOutput {
    @required
    item: ItemDetails
}

/// Replaces the editable fields; omitting description clears it.
@idempotent
@http(method: "PUT", uri: "/items/{itemId}", code: 200)
operation UpdateItem {
    input: UpdateItemInput
    output: UpdateItemOutput
    errors: [ResourceNotFoundException]
}

@input
structure UpdateItemInput {
    @required
    @httpLabel
    itemId: ItemId
    @required
    name: ItemName
    description: ItemDescription
}

@output
structure UpdateItemOutput {
    @required
    item: ItemDetails
}

/// Deletes an item; an already absent item also succeeds.
@idempotent
@http(method: "DELETE", uri: "/items/{itemId}", code: 204)
operation DeleteItem { input: DeleteItemInput }

@input
structure DeleteItemInput {
    @required
    @httpLabel
    itemId: ItemId
}

@readonly
@documentation("Lists items in identifier order with opaque pagination tokens.")
@paginated(inputToken: "nextToken", outputToken: "nextToken", pageSize: "maxResults", items: "items")
@http(method: "GET", uri: "/items", code: 200)
operation ListItems {
    input: ListItemsInput
    output: ListItemsOutput
}

@input
structure ListItemsInput {
    @httpQuery("nextToken")
    nextToken: NextToken
    @httpQuery("maxResults")
    @range(min: 1, max: 100)
    maxResults: Integer
}

@output
structure ListItemsOutput {
    @required
    items: ItemList
    nextToken: NextToken
}

enum ResourceType {
    ITEM = "Item"
}

@error("client")
@documentation("The requested item does not exist.")
@httpError(404)
structure ResourceNotFoundException {
    @required
    message: String
    @required
    resourceId: ItemId
    @required
    resourceType: ResourceType
}

@error("client")
@documentation("A client token was reused with different item fields.")
@httpError(409)
structure ConflictException {
    @required
    message: String
    @required
    resourceId: ItemId
    @required
    resourceType: ResourceType
}

@error("client")
@documentation("The in-memory item or create-receipt limit was reached.")
@httpError(402)
structure ServiceQuotaExceededException {
    @required
    message: String
    @required
    resourceType: ResourceType
}
