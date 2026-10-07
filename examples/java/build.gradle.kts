plugins {
    application
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("example:myservice-sdk:0.1.0")
}

application {
    mainClass.set("example.MyServiceExample")
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(17)
    options.encoding = "UTF-8"
}
