plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.blinc.{{project_name_snake}}"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.blinc.{{project_name_snake}}"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }

    sourceSets {
        getByName("main") {
            // Include Rust-built native libraries
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.11.0")
}

// Copy Rust libraries to jniLibs before build
tasks.register<Copy>("copyRustLibs") {
    description = "Copy Rust-built libraries to jniLibs"
    group = "rust"

    val rustTargetDir = file("../../../../target")
    val jniLibsDir = file("src/main/jniLibs")

    val archMapping = mapOf(
        "aarch64-linux-android" to "arm64-v8a",
        "armv7-linux-androideabi" to "armeabi-v7a",
        "x86_64-linux-android" to "x86_64",
        "i686-linux-android" to "x86"
    )

    archMapping.forEach { (rustTarget, androidAbi) ->
        from("$rustTargetDir/$rustTarget/debug") {
            include("lib{{project_name_snake}}.so")
            into(androidAbi)
        }
        from("$rustTargetDir/$rustTarget/release") {
            include("lib{{project_name_snake}}.so")
            into(androidAbi)
        }
    }

    into(jniLibsDir)
}

tasks.named("preBuild") {
    dependsOn("copyRustLibs")
}
