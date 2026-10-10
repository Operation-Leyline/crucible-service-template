package example

import example.myservice.ClientConfig
import example.myservice.MyServiceClient

fun main(args: Array<String>) {
    val config = ClientConfig().apply {
        endpoint = args.firstOrNull() ?: "http://127.0.0.1:8080"
    }
    val client = MyServiceClient(config)
    val created = client.createItem {
        it.name("Kotlin client example")
            .notes("Created through the generated Java SDK from Kotlin")
    }
    val itemId = created.item().itemId()
    println("Created $itemId")
    var failure: Throwable? = null
    try {
        val fetched = client.getItem { it.itemId(itemId) }
        println("Read: ${fetched.item().name()}")
        val updated = client.updateItem { it.itemId(itemId).name("Updated by the Kotlin client") }
        println("Updated: ${updated.item().name()}")
        client.listItemsPages { it.maxResults(1) }.forEach { page ->
            for (item in page.items()) {
                println("Listed: ${item.name()} (${item.itemId()})")
            }
        }
    } catch (error: Throwable) {
        failure = error
        throw error
    } finally {
        try {
            client.deleteItem { it.itemId(itemId) }
            println("Deleted $itemId")
        } catch (cleanupError: Throwable) {
            failure?.addSuppressed(cleanupError) ?: throw cleanupError
        }
    }
}
