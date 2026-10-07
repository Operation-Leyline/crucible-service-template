package example;

import example.myservice.ClientConfig;
import example.myservice.MyServiceClient;
import example.myservice.model.CreateItemInput;
import example.myservice.model.DeleteItemInput;
import example.myservice.model.GetItemInput;
import example.myservice.model.ListItemsInput;
import example.myservice.model.UpdateItemInput;

public final class MyServiceExample {
    public static void main(String[] args) {
        var config = new ClientConfig();
        config.endpoint = args.length == 0 ? "http://127.0.0.1:8080" : args[0];
        var client = new MyServiceClient(config);
        var created = client.createItem(CreateItemInput.builder()
            .name("Java client example")
            .description("Created through the generated Java SDK")
            .build());
        var itemId = created.item().itemId();
        System.out.println("Created " + itemId);
        try {
            var fetched = client.getItem(GetItemInput.builder().itemId(itemId).build());
            System.out.println("Read: " + fetched.item().name());
            var updated = client.updateItem(UpdateItemInput.builder()
                .itemId(itemId).name("Updated by the Java client").build());
            System.out.println("Updated: " + updated.item().name());
            var pages = client.listItemsPages(ListItemsInput.builder().maxResults(1).build());
            while (pages.hasNext()) {
                for (var item : pages.next().items()) {
                    System.out.println("Listed: " + item.name() + " (" + item.itemId() + ")");
                }
            }
        } finally {
            client.deleteItem(DeleteItemInput.builder().itemId(itemId).build());
            System.out.println("Deleted " + itemId);
        }
    }
}
