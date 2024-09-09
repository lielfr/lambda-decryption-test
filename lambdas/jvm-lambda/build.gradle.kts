plugins {
    id("java")
}

group = "com.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.amazonaws:aws-lambda-java-core:1.2.3")
    implementation("com.amazonaws:aws-lambda-java-events:3.11.6")
    implementation("com.amazonaws:aws-lambda-java-log4j2:1.6.0")
    implementation(platform("software.amazon.awssdk:bom:2.16.1"))
    implementation("software.amazon.awssdk:s3:2.26.16")
    implementation("software.amazon.awssdk:secretsmanager:2.26.16")
    testImplementation(platform("org.junit:junit-bom:5.10.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
}

tasks.test {
    useJUnitPlatform()
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

task("packageJar", type = Zip::class) {
    into("lib") {
        from(tasks.jar.get().archiveFile)
        from(configurations.runtimeClasspath)
    }
}