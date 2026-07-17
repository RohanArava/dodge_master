#!/bin/zsh

# Grab PROJECT from Cargo.toml project.name
PROJECT=$(grep "name" Cargo.toml | cut -d '"' -f 2 | head -n 1)
PROJECT_ORG="bevyengine"
PROJECT_ORG_STRUCTURE="org.${PROJECT_ORG}.${PROJECT}"
BEVY_RELEASE="refs/heads/release-0.15.2"

echo "Creating new Bevy project: ${PROJECT} with organization structure: ${PROJECT_ORG_STRUCTURE}"

mkdir -p app/src/main/cpp/
mkdir -p app/src/main/java/org/${PROJECT_ORG}/${PROJECT}/
mkdir -p assets/android-res/mipmap-mdpi/
mkdir -p assets/sounds/
mkdir -p gradle


# -------------------------------------Clone from Github-------------------------------------
clone_file() {
    local file_path=$1
    local url=$2
    if [ ! -f "$file_path" ]; then
        echo "Cloning $(basename "$file_path")"
        curl -o "$file_path" -sL "$url"
        sleep 0.2
    fi
}

echo "Cloning files if they don't exist"
sleep 2
clone_file "gradle.properties" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/gradle.properties"
clone_file "build.gradle" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/build.gradle"
clone_file "gradle/libs.versions.toml" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/gradle/libs.versions.toml"
clone_file "app/CMakeLists.txt" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/app/CMakeLists.txt"
clone_file "app/src/main/cpp/dummy.cpp" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/app/src/main/cpp/dummy.cpp"
clone_file "settings.gradle" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/settings.gradle"
clone_file "app/build.gradle" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/app/build.gradle"
clone_file "app/src/main/AndroidManifest.xml" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/app/src/main/AndroidManifest.xml"
clone_file "app/src/main/java/org/${PROJECT_ORG}/${PROJECT}/MainActivity.java" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/examples/mobile/android_example/app/src/main/java/org/bevyengine/example/MainActivity.java"
clone_file "assets/android-res/mipmap-mdpi/ic_launcher.png" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/assets/android-res/mipmap-mdpi/ic_launcher.png"
clone_file "assets/sounds/breakout_collision.ogg" "https://raw.githubusercontent.com/bevyengine/bevy/${BEVY_RELEASE}/assets/sounds/breakout_collision.ogg"

# -------------------------------------CARGO.toml-------------------------------------
echo "Setting up Cargo.toml"

CARGO="Cargo.toml"

# Append the [lib] section if it doesn't already exist.
if ! grep -q "^\[lib\]" "$CARGO"; then
    cat << EOF >> "$CARGO"

[lib]
name = "$PROJECT"
path = "src/lib.rs"
crate-type = [
    "staticlib",
    "cdylib",    # needed for Android
    "rlib",      # rlib needed for running locally
]
EOF
fi

# Append the [[bin]] section if it doesn't already exist.
if ! grep -q "^\[\[bin\]\]" "$CARGO"; then
    cat << EOF >> "$CARGO"

[[bin]]
name = "$PROJECT"
path = "src/main.rs"
EOF
fi

# -------------------------------------modifications-------------------------------------
# In settings.gradle, replace rootProject.name .+ with rootProject.name = 'bevy_breakout_15' using the variables above
echo "Replacing rootProject.name in settings.gradle with ${PROJECT}"
sed -i '' "s/rootProject.name .*/rootProject.name = '${PROJECT}'/" settings.gradle


# In app/build.gradle, replace namespace and applicationId with the PROJECT_ORG_STRUCTURE
echo "Replacing namespace and applicationId in app/build.gradle with ${PROJECT_ORG_STRUCTURE}"
sed -i '' "s/org.bevyengine.example/${PROJECT_ORG_STRUCTURE}/" app/build.gradle

# Replace ../../../../ with ../ in app/build.gradle
echo "Replacing ../../../../ with ../ in app/build.gradle"
sed -i '' "s/..\/..\/..\/..\//..\//" app/build.gradle

# Replace org.bevyengine.example in app/src/main/java/org/bevyengine/example/MainActivity.java
echo "Replacing org.bevyengine.example in app/src/main/java/org/bevyengine/example/MainActivity.java with ${PROJECT_ORG_STRUCTURE}"
sed -i '' "s/org.bevyengine.example/${PROJECT_ORG_STRUCTURE}/" app/src/main/java/org/${PROJECT_ORG}/${PROJECT}/MainActivity.java

# then replace bevy_mobile_example with PROJECT
echo "Replacing bevy_mobile_example in app/src/main/java/org/bevyengine/example/MainActivity.java with ${PROJECT}"
sed -i '' "s/bevy_mobile_example/${PROJECT}/" app/src/main/java/org/${PROJECT_ORG}/${PROJECT}/MainActivity.java

# Update app/src/main/AndroidManifest.xml
echo "Replacing bevy_mobile_example in app/src/main/AndroidManifest.xml with ${PROJECT}"
sed -i '' "s/bevy_mobile_example/${PROJECT}/" app/src/main/AndroidManifest.xml
echo "Replacing Bevy Example in app/src/main/AndroidManifest.xml with ${PROJECT}"
sed -i '' "s/Bevy Example/${PROJECT}/" app/src/main/AndroidManifest.xml

# -------------------------------------Gradle-------------------------------------
if [ ! -f "./gradlew" ]; then
    echo "Setting up gradle by calling: gradle wrapper"
    sleep 0.5
    gradle wrapper
fi

# -------------------------------------Cargo NDK and jniLibs-------------------------------------
echo "Installing cargo-ndk"
sleep 1
cargo install --locked cargo-ndk  

echo "Building the jniLibs with cargo-ndk"
sleep 1
cargo ndk -t arm64-v8a -o app/src/main/jniLibs build --package $PROJECT

