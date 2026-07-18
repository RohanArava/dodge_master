./scripts/compile.sh
./gradlew assembleDebug
adb install --incremental app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.melodev.dodgemaster/com.melodev.dodgemaster.MainActivity
sleep 1s
adb logcat --pid=$(adb shell pidof -s com.melodev.dodgemaster) | grep -i "RustStdout"