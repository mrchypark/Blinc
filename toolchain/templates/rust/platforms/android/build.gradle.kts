// Top-level build file for {{project_name}}
plugins {
    id("com.android.application") version "8.2.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.22" apply false
}

// Task to build the Rust library for Android
tasks.register("buildRust") {
    description = "Build Rust library for Android"
    group = "rust"

    doLast {
        val architectures = listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
        val targets = mapOf(
            "arm64-v8a" to "aarch64-linux-android",
            "armeabi-v7a" to "armv7-linux-androideabi",
            "x86_64" to "x86_64-linux-android",
            "x86" to "i686-linux-android"
        )

        // Build for default architecture (arm64-v8a) in debug
        val arch = "arm64-v8a"
        val target = targets[arch]!!

        exec {
            workingDir = file("../..")
            commandLine("cargo", "ndk", "-t", arch, "build", "--lib")
        }
    }
}

tasks.register("buildRustRelease") {
    description = "Build Rust library for Android (release)"
    group = "rust"

    doLast {
        val architectures = listOf("arm64-v8a")
        val targets = mapOf(
            "arm64-v8a" to "aarch64-linux-android"
        )

        for (arch in architectures) {
            exec {
                workingDir = file("../..")
                commandLine("cargo", "ndk", "-t", arch, "build", "--lib", "--release")
            }
        }
    }
}
