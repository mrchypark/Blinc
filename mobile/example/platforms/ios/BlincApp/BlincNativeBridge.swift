/**
 * Blinc Native Bridge for iOS
 *
 * Swift implementation for handling native calls from Rust.
 * Register handlers for each namespace/function, then Rust can call
 * them via native_call("namespace", "function", args).
 *
 * Usage:
 * ```swift
 * // In AppDelegate.application(_:didFinishLaunchingWithOptions:)
 * BlincNativeBridge.shared.registerDefaults()
 * BlincNativeBridge.shared.connectToRust()
 *
 * // Or register custom handlers
 * BlincNativeBridge.shared.register(namespace: "myapi", name: "my_function") { args in
 *     // args is [Any]
 *     return "result"
 * }
 * ```
 */

import Foundation
import UIKit
import AudioToolbox
import AVFoundation
import CoreBluetooth
import CoreLocation
import CoreMotion
import Photos
import UserNotifications

public final class BlincNativeBridge {
    private struct PermissionCapabilityState {
        let status: String
        let canRequest: Bool
        let requiresSettingsRedirect: Bool
        let supported: Bool
    }


    public static let shared = BlincNativeBridge()

    // Handler type: (args: [Any]) throws -> Any?
    private var handlers: [String: [String: ([Any]) throws -> Any?]] = [:]
    private let sensorCollector = IOSSensorCollector()
    private let bleCollector = IOSBleCollector()
    private let notificationCapabilityLock = NSLock()
    private var cachedNotificationCapability: PermissionCapabilityState?

    private init() {}

    // MARK: - Registration

    /// Register a native function handler
    ///
    /// - Parameters:
    ///   - namespace: The namespace (e.g., "device", "haptics")
    ///   - name: The function name
    ///   - handler: Handler that receives args array and returns a result
    public func register(namespace: String, name: String, handler: @escaping ([Any]) throws -> Any?) {
        if handlers[namespace] == nil {
            handlers[namespace] = [:]
        }
        handlers[namespace]![name] = handler
    }

    /// Convenience: Register a no-arg function returning String
    public func registerString(namespace: String, name: String, handler: @escaping () -> String) {
        register(namespace: namespace, name: name) { _ in handler() }
    }

    /// Convenience: Register a no-arg void function
    public func registerVoid(namespace: String, name: String, handler: @escaping () -> Void) {
        register(namespace: namespace, name: name) { _ in handler(); return nil }
    }

    // MARK: - Native Call Handler

    /// Called from Rust via C FFI to execute a registered function
    ///
    /// - Parameters:
    ///   - namespace: The namespace
    ///   - name: The function name
    ///   - argsJson: JSON-encoded arguments array
    /// - Returns: JSON-encoded result or error
    func callNative(namespace: String, name: String, argsJson: String) -> String {
        do {
            guard let nsHandlers = handlers[namespace] else {
                return errorJson(type: "NotRegistered", message: "Namespace '\(namespace)' not found")
            }

            guard let handler = nsHandlers[name] else {
                return errorJson(type: "NotRegistered", message: "Function '\(namespace).\(name)' not found")
            }

            // Parse args from JSON
            let args = parseArgs(argsJson)

            // Call handler
            let result = try handler(args)

            return successJson(value: result)
        } catch {
            return errorJson(type: "PlatformError", message: error.localizedDescription)
        }
    }

    /// Connect to Rust by registering our native call function
    public func connectToRust() {
        blinc_set_native_call_fn(blinc_ios_native_call)
    }

    // MARK: - Default Handlers

    /// Register default handlers for common functionality
    public func registerDefaults() {

        // =====================================================================
        // Device namespace
        // =====================================================================

        registerString(namespace: "device", name: "get_battery_level") {
            UIDevice.current.isBatteryMonitoringEnabled = true
            let level = UIDevice.current.batteryLevel
            UIDevice.current.isBatteryMonitoringEnabled = false
            return level >= 0 ? String(Int(level * 100)) : "0"
        }

        registerString(namespace: "device", name: "get_model") {
            UIDevice.current.model
        }

        registerString(namespace: "device", name: "get_os_version") {
            UIDevice.current.systemVersion
        }

        register(namespace: "device", name: "is_low_power_mode") { _ in
            ProcessInfo.processInfo.isLowPowerModeEnabled
        }

        register(namespace: "device", name: "has_notch") { _ in
            if #available(iOS 11.0, *) {
                let window = UIApplication.shared.windows.first
                return (window?.safeAreaInsets.top ?? 0) > 20
            }
            return false
        }

        registerString(namespace: "device", name: "get_locale") {
            Locale.current.identifier
        }

        registerString(namespace: "device", name: "get_timezone") {
            TimeZone.current.identifier
        }

        // =====================================================================
        // Haptics namespace
        // =====================================================================

        register(namespace: "haptics", name: "vibrate") { args in
            // iOS doesn't support custom duration vibration via public API
            AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)
            return nil
        }

        register(namespace: "haptics", name: "impact") { args in
            if #available(iOS 10.0, *) {
                let style: Int = args.first as? Int ?? 1
                let feedbackStyle: UIImpactFeedbackGenerator.FeedbackStyle
                switch style {
                case 0: feedbackStyle = .light
                case 2: feedbackStyle = .heavy
                default: feedbackStyle = .medium
                }
                let generator = UIImpactFeedbackGenerator(style: feedbackStyle)
                generator.prepare()
                generator.impactOccurred()
            }
            return nil
        }

        registerVoid(namespace: "haptics", name: "selection") {
            if #available(iOS 10.0, *) {
                let generator = UISelectionFeedbackGenerator()
                generator.prepare()
                generator.selectionChanged()
            }
        }

        registerVoid(namespace: "haptics", name: "success") {
            if #available(iOS 10.0, *) {
                let generator = UINotificationFeedbackGenerator()
                generator.prepare()
                generator.notificationOccurred(.success)
            }
        }

        registerVoid(namespace: "haptics", name: "warning") {
            if #available(iOS 10.0, *) {
                let generator = UINotificationFeedbackGenerator()
                generator.prepare()
                generator.notificationOccurred(.warning)
            }
        }

        registerVoid(namespace: "haptics", name: "error") {
            if #available(iOS 10.0, *) {
                let generator = UINotificationFeedbackGenerator()
                generator.prepare()
                generator.notificationOccurred(.error)
            }
        }

        // =====================================================================
        // Clipboard namespace
        // =====================================================================

        register(namespace: "clipboard", name: "copy") { args in
            let text = args.first as? String ?? ""
            UIPasteboard.general.string = text
            return nil
        }

        registerString(namespace: "clipboard", name: "paste") {
            UIPasteboard.general.string ?? ""
        }

        register(namespace: "clipboard", name: "has_content") { _ in
            UIPasteboard.general.hasStrings
        }

        registerVoid(namespace: "clipboard", name: "clear") {
            UIPasteboard.general.items = []
        }

        // =====================================================================
        // App namespace
        // =====================================================================

        registerString(namespace: "app", name: "get_version") {
            Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
        }

        registerString(namespace: "app", name: "get_build_number") {
            Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"
        }

        registerString(namespace: "app", name: "get_bundle_id") {
            Bundle.main.bundleIdentifier ?? ""
        }

        register(namespace: "app", name: "open_url") { args in
            guard let urlString = args.first as? String,
                  let url = URL(string: urlString) else {
                return false
            }

            if #available(iOS 10.0, *) {
                UIApplication.shared.open(url, options: [:], completionHandler: nil)
                return true
            } else {
                return UIApplication.shared.openURL(url)
            }
        }

        register(namespace: "app", name: "share_text") { args in
            let text = args.first as? String ?? ""
            DispatchQueue.main.async {
                let activityVC = UIActivityViewController(activityItems: [text], applicationActivities: nil)
                if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                   let rootVC = windowScene.windows.first?.rootViewController {
                    rootVC.present(activityVC, animated: true)
                }
            }
            return nil
        }

        // =====================================================================
        // Permissions namespace
        // =====================================================================

        register(namespace: "permissions", name: "has_location") { _ in
            self.permissionCapabilityPayload(self.locationPermissionCapability(always: false))
        }
        register(namespace: "permissions", name: "has_location_always") { _ in
            self.permissionCapabilityPayload(self.locationPermissionCapability(always: true))
        }
        register(namespace: "permissions", name: "has_motion") { _ in
            self.permissionCapabilityPayload(self.motionPermissionCapability())
        }
        register(namespace: "permissions", name: "has_camera") { _ in
            self.permissionCapabilityPayload(self.cameraPermissionCapability())
        }
        register(namespace: "permissions", name: "has_microphone") { _ in
            self.permissionCapabilityPayload(self.microphonePermissionCapability())
        }
        register(namespace: "permissions", name: "has_photos") { _ in
            self.permissionCapabilityPayload(self.photosPermissionCapability())
        }
        register(namespace: "permissions", name: "has_notifications") { _ in
            self.permissionCapabilityPayload(self.notificationsPermissionCapability())
        }
        register(namespace: "permissions", name: "has_bluetooth_scan") { _ in
            self.permissionCapabilityPayload(self.bluetoothPermissionCapability())
        }
        register(namespace: "permissions", name: "has_bluetooth_connect") { _ in
            self.permissionCapabilityPayload(self.bluetoothPermissionCapability())
        }
        register(namespace: "permissions", name: "request_location_when_in_use") { _ in
            let previous = self.locationPermissionCapability(always: false)
            _ = self.sensorCollector.requestLocationPermissionWhenInUse()
            let current = self.locationPermissionCapability(always: false)
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_location_always") { _ in
            let previous = self.locationPermissionCapability(always: true)
            _ = self.sensorCollector.requestLocationPermissionAlways()
            let current = self.locationPermissionCapability(always: true)
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_motion") { _ in
            let previous = self.motionPermissionCapability()
            _ = self.sensorCollector.requestMotionPermission()
            let current = self.motionPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_camera") { _ in
            let previous = self.cameraPermissionCapability()
            guard self.requestCameraPermission() != nil else {
                return self.permissionRequestTimedOutPayload(previous: previous)
            }
            let current = self.cameraPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_microphone") { _ in
            let previous = self.microphonePermissionCapability()
            guard self.requestMicrophonePermission() != nil else {
                return self.permissionRequestTimedOutPayload(previous: previous)
            }
            let current = self.microphonePermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_photos") { _ in
            let previous = self.photosPermissionCapability()
            guard self.requestPhotosPermission() != nil else {
                return self.permissionRequestTimedOutPayload(previous: previous)
            }
            let current = self.photosPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_notifications") { _ in
            let previous = self.notificationsPermissionCapability()
            guard self.requestNotificationsPermission() != nil else {
                return self.permissionRequestTimedOutPayload(previous: previous)
            }
            let current = self.notificationsPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_bluetooth_scan") { _ in
            let previous = self.bluetoothPermissionCapability()
            _ = self.bleCollector.requestBluetoothPermission()
            let current = self.bluetoothPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "request_bluetooth_connect") { _ in
            let previous = self.bluetoothPermissionCapability()
            _ = self.bleCollector.requestBluetoothPermission()
            let current = self.bluetoothPermissionCapability()
            return self.permissionRequestPayload(previous: previous, current: current)
        }
        register(namespace: "permissions", name: "open_settings") { _ in
            self.openApplicationSettings()
        }

        // =====================================================================
        // BLE namespace
        // =====================================================================

        register(namespace: "ble", name: "configure") { args in
            let configJson = args.first as? String ?? "{}"
            return self.bleCollector.configure(configJson: configJson)
        }

        register(namespace: "ble", name: "start") { args in
            let sessionId = args.first as? String ?? ""
            return self.bleCollector.start(sessionId: sessionId)
        }

        register(namespace: "ble", name: "stop") { args in
            let sessionId = args.first as? String ?? ""
            return self.bleCollector.stop(sessionId: sessionId)
        }

        registerString(namespace: "ble", name: "status") {
            self.bleCollector.statusJson()
        }

        register(namespace: "ble", name: "drain_results") { args in
            let maxResults = (args.first as? NSNumber)?.intValue ?? 64
            return self.bleCollector.drainResults(maxResults: maxResults)
        }

        // =====================================================================
        // Sensor namespace (default stubs)
        // =====================================================================

        register(namespace: "sensor", name: "configure") { args in
            let configJson = args.first as? String ?? "{}"
            return self.sensorCollector.configure(configJson: configJson)
        }

        register(namespace: "sensor", name: "start") { args in
            let sessionId = args.first as? String ?? ""
            return self.sensorCollector.start(sessionId: sessionId)
        }

        register(namespace: "sensor", name: "stop") { args in
            let sessionId = args.first as? String ?? ""
            return self.sensorCollector.stop(sessionId: sessionId)
        }

        registerString(namespace: "sensor", name: "status") {
            self.sensorCollector.statusJson()
        }

        register(namespace: "sensor", name: "drain_frames") { args in
            let maxFrames = (args.first as? NSNumber)?.intValue ?? 64
            return self.sensorCollector.drainFrames(maxFrames: maxFrames)
        }

        register(namespace: "sensor", name: "peek_frames") { args in
            let maxFrames = (args.first as? NSNumber)?.intValue ?? 32
            return self.sensorCollector.peekFrames(maxFrames: maxFrames)
        }

        registerString(namespace: "sensor", name: "supported_kinds") {
            self.sensorCollector.supportedKindsJson()
        }

        registerVoid(namespace: "sensor", name: "clear_buffer") {
            self.sensorCollector.clearBuffer()
        }
    }

    private func hasMicrophonePermission() -> Bool {
        let permission = AVAudioSession.sharedInstance().recordPermission
        return permission == .granted
    }

    private func hasCameraPermission() -> Bool {
        AVCaptureDevice.authorizationStatus(for: .video) == .authorized
    }

    private func requestCameraPermission() -> Bool? {
        if hasCameraPermission() {
            return true
        }
        return waitForAsyncPermissionDecision { resolve in
            AVCaptureDevice.requestAccess(for: .video) { allowed in
                resolve(allowed)
            }
        }
    }

    private func requestMicrophonePermission() -> Bool? {
        if hasMicrophonePermission() {
            return true
        }
        return waitForAsyncPermissionDecision { resolve in
            AVAudioSession.sharedInstance().requestRecordPermission { allowed in
                resolve(allowed)
            }
        }
    }

    private func requestPhotosPermission() -> Bool? {
        let currentCapability = photosPermissionCapability()
        if currentCapability.status == "granted" || currentCapability.status == "limited" {
            return true
        }
        return waitForAsyncPermissionDecision { resolve in
            if #available(iOS 14.0, *) {
                PHPhotoLibrary.requestAuthorization(for: .readWrite) { status in
                    resolve(status == .authorized || status == .limited)
                }
            } else {
                PHPhotoLibrary.requestAuthorization { status in
                    resolve(status == .authorized)
                }
            }
        }
    }

    private func requestNotificationsPermission() -> Bool? {
        let capability = notificationsPermissionCapability()
        if capability.status == "granted" || capability.status == "provisional" {
            return true
        }
        return waitForAsyncPermissionDecision { resolve in
            UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .badge, .sound]) { allowed, _ in
                resolve(allowed)
            }
        }
    }

    // MARK: - Helper Functions

    private func parseArgs(_ json: String) -> [Any] {
        guard let data = json.data(using: .utf8),
              let array = try? JSONSerialization.jsonObject(with: data) as? [Any] else {
            return []
        }
        return array
    }

    private func successJson(value: Any?) -> String {
        var result: [String: Any] = ["success": true]

        switch value {
        case nil:
            result["value"] = NSNull()
        case let bool as Bool:
            result["value"] = bool
        case let int as Int:
            result["value"] = int
        case let int64 as Int64:
            result["value"] = int64
        case let float as Float:
            result["value"] = float
        case let double as Double:
            result["value"] = double
        case let string as String:
            result["value"] = string
        case let data as Data:
            result["value"] = data.base64EncodedString()
        case let dictionary as [String: Any]:
            result["value"] = dictionary
        case let array as [Any]:
            result["value"] = array
        default:
            result["value"] = String(describing: value)
        }

        if let data = try? JSONSerialization.data(withJSONObject: result),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"success\":true,\"value\":null}"
    }

    private func errorJson(type: String, message: String) -> String {
        let result: [String: Any] = [
            "success": false,
            "errorType": type,
            "errorMessage": message
        ]

        if let data = try? JSONSerialization.data(withJSONObject: result),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"success\":false,\"errorType\":\"\(type)\",\"errorMessage\":\"\(message)\"}"
    }

    private func openApplicationSettings() -> Bool {
        guard let url = URL(string: UIApplication.openSettingsURLString) else {
            return false
        }
        if Thread.isMainThread {
            UIApplication.shared.open(url, options: [:], completionHandler: nil)
            return true
        }
        DispatchQueue.main.async {
            UIApplication.shared.open(url, options: [:], completionHandler: nil)
        }
        return true
    }

    private func permissionCapabilityPayload(_ capability: PermissionCapabilityState) -> [String: Any] {
        [
            "status": capability.status,
            "canRequest": capability.canRequest,
            "requiresSettingsRedirect": capability.requiresSettingsRedirect,
            "supported": capability.supported,
        ]
    }

    private func permissionRequestPayload(
        previous: PermissionCapabilityState,
        current: PermissionCapabilityState
    ) -> [String: Any] {
        let effectiveCurrent: PermissionCapabilityState
        if current.status == previous.status && previous.canRequest {
            effectiveCurrent = previous
        } else {
            effectiveCurrent = current
        }
        [
            "status": effectiveCurrent.status,
            "previousStatus": previous.status,
            "canRequestAgain": effectiveCurrent.canRequest,
            "requiresSettingsRedirect": effectiveCurrent.requiresSettingsRedirect,
        ]
    }

    private func permissionRequestTimedOutPayload(previous: PermissionCapabilityState) -> [String: Any] {
        [
            "status": "pending",
            "previousStatus": previous.status,
            "canRequestAgain": previous.canRequest,
            "requiresSettingsRedirect": false,
            "deferred": true,
        ]
    }

    private func waitForAsyncPermissionDecision(
        timeout: TimeInterval = 30.0,
        work: (@escaping (Bool) -> Void) -> Void
    ) -> Bool? {
        let lock = NSLock()
        var result: Bool?
        let semaphore = DispatchSemaphore(value: 0)

        let resolve: (Bool) -> Void = { value in
            lock.lock()
            result = value
            lock.unlock()
            semaphore.signal()
        }

        if Thread.isMainThread {
            work(resolve)
        } else {
            DispatchQueue.main.async {
                work(resolve)
            }
            _ = semaphore.wait(timeout: .now() + timeout)
            lock.lock()
            let current = result
            lock.unlock()
            if let current {
                return current
            }
        }

        return nil
    }

    private func waitForNotificationPermissionCapability(timeout: TimeInterval = 2.0) -> PermissionCapabilityState {
        if Thread.isMainThread {
            if let cached = currentCachedNotificationCapability() {
                return cached
            }
            UNUserNotificationCenter.current().getNotificationSettings { settings in
                self.storeNotificationCapability(
                    self.notificationPermissionCapability(settings.authorizationStatus)
                )
            }
            return pendingNotificationCapability()
        }

        let lock = NSLock()
        var capability: PermissionCapabilityState?
        let semaphore = DispatchSemaphore(value: 0)

        UNUserNotificationCenter.current().getNotificationSettings { settings in
            let resolved = self.notificationPermissionCapability(settings.authorizationStatus)
            self.storeNotificationCapability(resolved)
            lock.lock()
            capability = resolved
            lock.unlock()
            semaphore.signal()
        }

        _ = semaphore.wait(timeout: .now() + timeout)
        lock.lock()
        let current = capability
        lock.unlock()
        if let current {
            return current
        }

        return unknownNotificationCapability()
    }

    private func currentCachedNotificationCapability() -> PermissionCapabilityState? {
        notificationCapabilityLock.lock()
        defer { notificationCapabilityLock.unlock() }
        return cachedNotificationCapability
    }

    private func storeNotificationCapability(_ capability: PermissionCapabilityState) {
        notificationCapabilityLock.lock()
        cachedNotificationCapability = capability
        notificationCapabilityLock.unlock()
    }

    private func unknownNotificationCapability() -> PermissionCapabilityState {
        PermissionCapabilityState(
            status: "unknown",
            canRequest: false,
            requiresSettingsRedirect: false,
            supported: true
        )
    }

    private func pendingNotificationCapability() -> PermissionCapabilityState {
        PermissionCapabilityState(
            status: "pending",
            canRequest: false,
            requiresSettingsRedirect: false,
            supported: true
        )
    }

    private func locationPermissionCapability(always: Bool) -> PermissionCapabilityState {
        switch self.sensorCollector.locationAuthorizationStatusValue() {
        case .authorizedAlways:
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .authorizedWhenInUse:
            if always {
                return PermissionCapabilityState(status: "denied", canRequest: true, requiresSettingsRedirect: false, supported: true)
            }
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .denied:
            return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .restricted:
            return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .notDetermined:
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        @unknown default:
            return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
        }
    }

    private func motionPermissionCapability() -> PermissionCapabilityState {
        if #available(iOS 11.0, *) {
            let activity = CMMotionActivityManager.authorizationStatus()
            let pedometer = CMPedometer.authorizationStatus()
            if activity == .authorized || pedometer == .authorized {
                return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
            }
            if activity == .restricted || pedometer == .restricted {
                return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
            }
            if activity == .denied || pedometer == .denied {
                return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
            }
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        }
        return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
    }

    private func microphonePermissionCapability() -> PermissionCapabilityState {
        switch AVAudioSession.sharedInstance().recordPermission {
        case .granted:
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .denied:
            return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .undetermined:
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        @unknown default:
            return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
        }
    }

    private func cameraPermissionCapability() -> PermissionCapabilityState {
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .denied:
            return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .restricted:
            return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .notDetermined:
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        @unknown default:
            return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
        }
    }

    private func photosPermissionCapability() -> PermissionCapabilityState {
        if #available(iOS 14.0, *) {
            switch PHPhotoLibrary.authorizationStatus(for: .readWrite) {
            case .authorized:
                return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
            case .limited:
                return PermissionCapabilityState(status: "limited", canRequest: false, requiresSettingsRedirect: false, supported: true)
            case .denied:
                return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
            case .restricted:
                return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
            case .notDetermined:
                return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
            @unknown default:
                return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
            }
        }

        switch PHPhotoLibrary.authorizationStatus() {
        case .authorized:
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .denied:
            return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .restricted:
            return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .notDetermined:
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        case .limited:
            return PermissionCapabilityState(status: "limited", canRequest: false, requiresSettingsRedirect: false, supported: true)
        @unknown default:
            return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
        }
    }

    private func notificationsPermissionCapability() -> PermissionCapabilityState {
        waitForNotificationPermissionCapability()
    }

    private func notificationPermissionCapability(_ status: UNAuthorizationStatus) -> PermissionCapabilityState {
        switch status {
        case .authorized:
            return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .provisional, .ephemeral:
            return PermissionCapabilityState(status: "provisional", canRequest: false, requiresSettingsRedirect: false, supported: true)
        case .denied:
            return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
        case .notDetermined:
            return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
        @unknown default:
            return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
        }
    }

    private func bluetoothPermissionCapability() -> PermissionCapabilityState {
        if #available(iOS 13.0, *) {
            switch CBManager.authorization {
            case .allowedAlways:
                return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
            case .denied:
                return PermissionCapabilityState(status: "denied", canRequest: false, requiresSettingsRedirect: true, supported: true)
            case .restricted:
                return PermissionCapabilityState(status: "restricted", canRequest: false, requiresSettingsRedirect: true, supported: true)
            case .notDetermined:
                return PermissionCapabilityState(status: "not_determined", canRequest: true, requiresSettingsRedirect: false, supported: true)
            @unknown default:
                return PermissionCapabilityState(status: "unknown", canRequest: false, requiresSettingsRedirect: false, supported: true)
            }
        }
        return PermissionCapabilityState(status: "granted", canRequest: false, requiresSettingsRedirect: false, supported: true)
    }
}

private struct IOSSensorConfig {
    var enabled: Set<String> = ["gps", "accelerometer", "gyroscope"]
    var gpsHz: Int = 1
    var imuHz: Int = 50
    var frameFlushMs: Int = 200
}

private final class IOSSensorCollector: NSObject, CLLocationManagerDelegate {
    private let lock = NSLock()
    private var frameBuffer: [[String: Any]] = []
    private var config = IOSSensorConfig()
    private var running = false
    private var activeSessionId: String?
    private var seq: UInt64 = 0
    private var lastGpsMonotonicNs: UInt64 = 0
    private let maxBufferedFrames = 4096

    private let motionManager = CMMotionManager()
    private let locationManager = CLLocationManager()
    private let callbackQueue: OperationQueue = {
        let queue = OperationQueue()
        queue.name = "com.blinc.sensors.ios.callbacks"
        queue.maxConcurrentOperationCount = 1
        queue.qualityOfService = .userInitiated
        return queue
    }()

    private var altimeter: CMAltimeter?
    private var pedometer: CMPedometer?
    private var activityManager: CMMotionActivityManager?
    private var permissionActivityManager: CMMotionActivityManager?
    private var permissionPedometer: CMPedometer?

    override init() {
        super.init()
        runOnMainSync {
            self.locationManager.delegate = self
            self.locationManager.desiredAccuracy = kCLLocationAccuracyBest
            self.locationManager.distanceFilter = kCLDistanceFilterNone
        }
    }

    func configure(configJson: String) -> Bool {
        guard let data = configJson.data(using: .utf8),
              let root = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
            return false
        }

        var enabled = Set<String>()
        if let enabledArray = root["enabled"] as? [Any] {
            for value in enabledArray {
                if let kind = value as? String {
                    let normalized = kind.trimmingCharacters(in: .whitespacesAndNewlines)
                    if !normalized.isEmpty {
                        enabled.insert(normalized)
                    }
                }
            }
        }

        let next = IOSSensorConfig(
            enabled: enabled.isEmpty ? IOSSensorConfig().enabled : enabled,
            gpsHz: intValue(root["gps_hz"], defaultValue: 1, minValue: 1),
            imuHz: intValue(root["imu_hz"], defaultValue: 50, minValue: 1),
            frameFlushMs: intValue(root["frame_flush_ms"], defaultValue: 200, minValue: 20)
        )

        lock.lock()
        config = next
        lock.unlock()
        return true
    }

    func start(sessionId: String) -> Bool {
        let normalizedId = sessionId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedId.isEmpty else { return false }

        stopAllStreams()
        lock.lock()
        running = true
        activeSessionId = normalizedId
        lock.unlock()

        startConfiguredStreams()
        return true
    }

    func stop(sessionId: String) -> Bool {
        let normalizedId = sessionId.trimmingCharacters(in: .whitespacesAndNewlines)

        lock.lock()
        if !running {
            lock.unlock()
            return true
        }
        if !normalizedId.isEmpty, normalizedId != activeSessionId {
            lock.unlock()
            return false
        }
        running = false
        activeSessionId = nil
        lock.unlock()

        stopAllStreams()
        return true
    }

    func statusJson() -> String {
        lock.lock()
        let payload: [String: Any] = [
            "running": running,
            "buffered_frames": frameBuffer.count,
            "active_session_id": activeSessionId ?? NSNull()
        ]
        lock.unlock()
        return toJsonString(payload, fallback: #"{"running":false,"buffered_frames":0,"active_session_id":null}"#)
    }

    func drainFrames(maxFrames: Int) -> String {
        let count = max(1, min(maxFrames, 2048))
        lock.lock()
        let drainedCount = min(count, frameBuffer.count)
        let drained = Array(frameBuffer.prefix(drainedCount))
        if drainedCount > 0 {
            frameBuffer.removeFirst(drainedCount)
        }
        lock.unlock()
        return toJsonString(drained, fallback: "[]")
    }

    func peekFrames(maxFrames: Int) -> String {
        let count = max(1, min(maxFrames, 256))
        lock.lock()
        let preview = Array(frameBuffer.suffix(count))
        lock.unlock()
        return toJsonString(preview, fallback: "[]")
    }

    func clearBuffer() {
        lock.lock()
        frameBuffer.removeAll(keepingCapacity: false)
        lock.unlock()
    }

    func supportedKindsJson() -> String {
        var kinds: [String] = []

        if CLLocationManager.locationServicesEnabled() {
            kinds.append("gps")
            if CLLocationManager.headingAvailable() {
                kinds.append("heading")
            }
            if CLLocationManager.significantLocationChangeMonitoringAvailable() {
                kinds.append("significant_motion")
            }
        }

        if motionManager.isAccelerometerAvailable {
            kinds.append("accelerometer")
        }
        if motionManager.isGyroAvailable {
            kinds.append("gyroscope")
        }
        if motionManager.isMagnetometerAvailable {
            kinds.append("magnetometer")
        }
        if motionManager.isDeviceMotionAvailable {
            kinds.append(contentsOf: [
                "linear_acceleration",
                "gravity",
                "rotation_vector",
                "quaternion",
                "device_motion"
            ])
        }
        if CMAltimeter.isRelativeAltitudeAvailable() {
            kinds.append("barometer")
        }
        if CMPedometer.isStepCountingAvailable() {
            kinds.append("step_counter")
        }
        if CMPedometer.isCadenceAvailable() {
            kinds.append("cadence")
        }
        if CMPedometer.isFloorCountingAvailable() {
            kinds.append("floor_climb")
        }
        if CMMotionActivityManager.isActivityAvailable() {
            kinds.append("activity")
        }

        return toJsonString(Array(Set(kinds)).sorted(), fallback: "[]")
    }

    func hasLocationPermission() -> Bool {
        let status = locationAuthorizationStatusValue()
        return status == .authorizedAlways || status == .authorizedWhenInUse
    }

    func hasLocationAlwaysPermission() -> Bool {
        return locationAuthorizationStatusValue() == .authorizedAlways
    }

    func hasMotionPermission() -> Bool {
        if #available(iOS 11.0, *) {
            return CMMotionActivityManager.authorizationStatus() == .authorized ||
                CMPedometer.authorizationStatus() == .authorized
        }
        return true
    }

    func requestLocationPermissionWhenInUse() -> Bool {
        guard CLLocationManager.locationServicesEnabled() else { return false }
        runOnMainSync {
            self.locationManager.requestWhenInUseAuthorization()
        }
        return hasLocationPermission()
    }

    func requestLocationPermissionAlways() -> Bool {
        guard CLLocationManager.locationServicesEnabled() else { return false }
        runOnMainSync {
            self.locationManager.requestAlwaysAuthorization()
        }
        return hasLocationPermission()
    }

    func requestMotionPermission() -> Bool {
        if #available(iOS 11.0, *) {
            let activityStatus = CMMotionActivityManager.authorizationStatus()
            let pedometerStatus = CMPedometer.authorizationStatus()
            if activityStatus == .denied || activityStatus == .restricted ||
                pedometerStatus == .denied || pedometerStatus == .restricted {
                return false
            }
            if activityStatus == .authorized || pedometerStatus == .authorized {
                return true
            }
        }

        let now = Date()
        if CMMotionActivityManager.isActivityAvailable() {
            let manager = CMMotionActivityManager()
            permissionActivityManager = manager
            manager.queryActivityStarting(
                from: now.addingTimeInterval(-60),
                to: now,
                to: callbackQueue
            ) { [weak self] _, _ in
                self?.permissionActivityManager = nil
            }
        }
        if CMPedometer.isStepCountingAvailable() {
            let pedometer = CMPedometer()
            permissionPedometer = pedometer
            pedometer.queryPedometerData(
                from: now.addingTimeInterval(-60),
                to: now
            ) { [weak self] _, _ in
                self?.permissionPedometer = nil
            }
        }
        return hasMotionPermission()
    }

    private func startConfiguredStreams() {
        let localConfig = currentConfig()
        let imuInterval = 1.0 / Double(max(localConfig.imuHz, 1))

        if localConfig.enabled.contains("accelerometer"), motionManager.isAccelerometerAvailable {
            motionManager.accelerometerUpdateInterval = imuInterval
            motionManager.startAccelerometerUpdates(to: callbackQueue) { [weak self] data, _ in
                guard let self, let data else { return }
                self.appendFrame(
                    kind: "accelerometer",
                    monotonicNs: self.motionTimestampToMonotonicNs(data.timestamp),
                    accuracy: "medium",
                    values: [
                        data.acceleration.x,
                        data.acceleration.y,
                        data.acceleration.z
                    ]
                )
            }
        }

        if localConfig.enabled.contains("gyroscope"), motionManager.isGyroAvailable {
            motionManager.gyroUpdateInterval = imuInterval
            motionManager.startGyroUpdates(to: callbackQueue) { [weak self] data, _ in
                guard let self, let data else { return }
                self.appendFrame(
                    kind: "gyroscope",
                    monotonicNs: self.motionTimestampToMonotonicNs(data.timestamp),
                    accuracy: "medium",
                    values: [
                        data.rotationRate.x,
                        data.rotationRate.y,
                        data.rotationRate.z
                    ]
                )
            }
        }

        if localConfig.enabled.contains("magnetometer"), motionManager.isMagnetometerAvailable {
            motionManager.magnetometerUpdateInterval = imuInterval
            motionManager.startMagnetometerUpdates(to: callbackQueue) { [weak self] data, _ in
                guard let self, let data else { return }
                self.appendFrame(
                    kind: "magnetometer",
                    monotonicNs: self.motionTimestampToMonotonicNs(data.timestamp),
                    accuracy: "medium",
                    values: [
                        data.magneticField.x,
                        data.magneticField.y,
                        data.magneticField.z
                    ]
                )
            }
        }

        let needsDeviceMotion =
            localConfig.enabled.contains("linear_acceleration") ||
            localConfig.enabled.contains("gravity") ||
            localConfig.enabled.contains("rotation_vector") ||
            localConfig.enabled.contains("quaternion") ||
            localConfig.enabled.contains("device_motion") ||
            localConfig.enabled.contains("heading")
        if needsDeviceMotion, motionManager.isDeviceMotionAvailable {
            motionManager.deviceMotionUpdateInterval = imuInterval
            motionManager.startDeviceMotionUpdates(to: callbackQueue) { [weak self] motion, _ in
                guard let self, let motion else { return }
                let monotonicNs = self.motionTimestampToMonotonicNs(motion.timestamp)

                if localConfig.enabled.contains("linear_acceleration") {
                    self.appendFrame(
                        kind: "linear_acceleration",
                        monotonicNs: monotonicNs,
                        accuracy: "medium",
                        values: [
                            motion.userAcceleration.x,
                            motion.userAcceleration.y,
                            motion.userAcceleration.z
                        ]
                    )
                }

                if localConfig.enabled.contains("gravity") {
                    self.appendFrame(
                        kind: "gravity",
                        monotonicNs: monotonicNs,
                        accuracy: "medium",
                        values: [
                            motion.gravity.x,
                            motion.gravity.y,
                            motion.gravity.z
                        ]
                    )
                }

                if localConfig.enabled.contains("rotation_vector") || localConfig.enabled.contains("quaternion") {
                    let q = motion.attitude.quaternion
                    if localConfig.enabled.contains("rotation_vector") {
                        self.appendFrame(
                            kind: "rotation_vector",
                            monotonicNs: monotonicNs,
                            accuracy: "medium",
                            values: [q.x, q.y, q.z, q.w]
                        )
                    }
                    if localConfig.enabled.contains("quaternion") {
                        self.appendFrame(
                            kind: "quaternion",
                            monotonicNs: monotonicNs,
                            accuracy: "medium",
                            values: [q.x, q.y, q.z, q.w]
                        )
                    }
                }

                if localConfig.enabled.contains("device_motion") {
                    self.appendFrame(
                        kind: "device_motion",
                        monotonicNs: monotonicNs,
                        accuracy: "medium",
                        values: [
                            motion.attitude.roll,
                            motion.attitude.pitch,
                            motion.attitude.yaw,
                            motion.rotationRate.x,
                            motion.rotationRate.y,
                            motion.rotationRate.z,
                            motion.userAcceleration.x,
                            motion.userAcceleration.y,
                            motion.userAcceleration.z
                        ]
                    )
                }

                if localConfig.enabled.contains("heading"), motion.heading >= 0 {
                    self.appendFrame(
                        kind: "heading",
                        monotonicNs: monotonicNs,
                        accuracy: "medium",
                        values: [motion.heading]
                    )
                }
            }
        }

        if localConfig.enabled.contains("barometer"), CMAltimeter.isRelativeAltitudeAvailable() {
            let altimeter = CMAltimeter()
            self.altimeter = altimeter
            altimeter.startRelativeAltitudeUpdates(to: callbackQueue) { [weak self] data, _ in
                guard let self, let data else { return }
                self.appendFrame(
                    kind: "barometer",
                    monotonicNs: self.currentMonotonicNs(),
                    accuracy: "medium",
                    values: [
                        data.pressure.doubleValue * 10.0,
                        data.relativeAltitude.doubleValue
                    ]
                )
            }
        }

        let needsPedometer =
            localConfig.enabled.contains("step_counter") ||
            localConfig.enabled.contains("cadence") ||
            localConfig.enabled.contains("floor_climb")
        if needsPedometer,
           CMPedometer.isStepCountingAvailable() ||
            CMPedometer.isCadenceAvailable() ||
            CMPedometer.isFloorCountingAvailable() {
            let pedometer = CMPedometer()
            self.pedometer = pedometer
            pedometer.startUpdates(from: Date()) { [weak self] data, _ in
                guard let self, let data else { return }
                let monotonicNs = self.currentMonotonicNs()

                if localConfig.enabled.contains("step_counter"), CMPedometer.isStepCountingAvailable() {
                    self.appendFrame(
                        kind: "step_counter",
                        monotonicNs: monotonicNs,
                        accuracy: "high",
                        values: [data.numberOfSteps.doubleValue]
                    )
                }

                if localConfig.enabled.contains("cadence"),
                   CMPedometer.isCadenceAvailable(),
                   let cadence = data.currentCadence?.doubleValue {
                    self.appendFrame(
                        kind: "cadence",
                        monotonicNs: monotonicNs,
                        accuracy: "high",
                        values: [cadence]
                    )
                }

                if localConfig.enabled.contains("floor_climb"), CMPedometer.isFloorCountingAvailable() {
                    self.appendFrame(
                        kind: "floor_climb",
                        monotonicNs: monotonicNs,
                        accuracy: "high",
                        values: [
                            data.floorsAscended?.doubleValue ?? 0.0,
                            data.floorsDescended?.doubleValue ?? 0.0
                        ]
                    )
                }
            }
        }

        if localConfig.enabled.contains("activity"), CMMotionActivityManager.isActivityAvailable() {
            let manager = CMMotionActivityManager()
            self.activityManager = manager
            manager.startActivityUpdates(to: callbackQueue) { [weak self] activity in
                guard let self, let activity else { return }
                self.appendFrame(
                    kind: "activity",
                    monotonicNs: self.currentMonotonicNs(),
                    accuracy: self.mapMotionConfidence(activity.confidence),
                    values: [
                        activity.stationary ? 1.0 : 0.0,
                        activity.walking ? 1.0 : 0.0,
                        activity.running ? 1.0 : 0.0,
                        activity.automotive ? 1.0 : 0.0,
                        activity.cycling ? 1.0 : 0.0,
                        activity.unknown ? 1.0 : 0.0
                    ]
                )
            }
        }

        if localConfig.enabled.contains("gps") ||
            localConfig.enabled.contains("heading") ||
            localConfig.enabled.contains("significant_motion") {
            startLocationServices(localConfig: localConfig)
        }
    }

    private func startLocationServices(localConfig: IOSSensorConfig) {
        guard CLLocationManager.locationServicesEnabled() else { return }
        guard hasLocationPermission() else { return }

        runOnMainSync {
            self.locationManager.startUpdatingLocation()

            if localConfig.enabled.contains("heading"), CLLocationManager.headingAvailable() {
                self.locationManager.headingFilter = kCLHeadingFilterNone
                self.locationManager.startUpdatingHeading()
            }

            if localConfig.enabled.contains("significant_motion"),
               CLLocationManager.significantLocationChangeMonitoringAvailable() {
                self.locationManager.startMonitoringSignificantLocationChanges()
            }
        }
    }

    private func stopAllStreams() {
        motionManager.stopAccelerometerUpdates()
        motionManager.stopGyroUpdates()
        motionManager.stopMagnetometerUpdates()
        motionManager.stopDeviceMotionUpdates()

        altimeter?.stopRelativeAltitudeUpdates()
        altimeter = nil

        pedometer?.stopUpdates()
        pedometer = nil

        activityManager?.stopActivityUpdates()
        activityManager = nil

        runOnMainSync {
            self.locationManager.stopUpdatingLocation()
            self.locationManager.stopUpdatingHeading()
            if CLLocationManager.significantLocationChangeMonitoringAvailable() {
                self.locationManager.stopMonitoringSignificantLocationChanges()
            }
        }

        lock.lock()
        lastGpsMonotonicNs = 0
        lock.unlock()
    }

    private func currentConfig() -> IOSSensorConfig {
        lock.lock()
        let current = config
        lock.unlock()
        return current
    }

    private func appendFrame(
        kind: String,
        monotonicNs: UInt64,
        accuracy: String,
        values: [Double],
        unixTimeMs: Int64? = nil
    ) {
        lock.lock()
        defer { lock.unlock() }

        guard running else { return }
        seq &+= 1
        let frame: [String: Any] = [
            "seq": NSNumber(value: seq),
            "sensor": kind,
            "time_monotonic_ns": NSNumber(value: monotonicNs),
            "time_unix_ms": NSNumber(value: unixTimeMs ?? monotonicToUnixMs(monotonicNs)),
            "accuracy": accuracy,
            "values": values
        ]
        if frameBuffer.count >= maxBufferedFrames {
            frameBuffer.removeFirst()
        }
        frameBuffer.append(frame)
    }

    private func appendLocationFrame(_ location: CLLocation) {
        let localConfig = currentConfig()
        let monotonicNs = locationToMonotonicNs(location)
        let unixTimeMs = Int64(location.timestamp.timeIntervalSince1970 * 1000.0)
        let minIntervalNs = UInt64(1_000_000_000 / max(localConfig.gpsHz, 1))

        lock.lock()
        let shouldEmit = monotonicNs >= lastGpsMonotonicNs &&
            (monotonicNs - lastGpsMonotonicNs >= minIntervalNs || lastGpsMonotonicNs == 0)
        if shouldEmit {
            lastGpsMonotonicNs = monotonicNs
        }
        lock.unlock()

        guard shouldEmit else { return }

        appendFrame(
            kind: "gps",
            monotonicNs: monotonicNs,
            accuracy: mapLocationAccuracy(location.horizontalAccuracy),
            values: [
                location.coordinate.latitude,
                location.coordinate.longitude,
                location.altitude,
                location.speed,
                location.course,
                location.horizontalAccuracy
            ],
            unixTimeMs: unixTimeMs
        )
    }

    private func appendHeadingFrame(_ heading: CLHeading) {
        appendFrame(
            kind: "heading",
            monotonicNs: currentMonotonicNs(),
            accuracy: mapHeadingAccuracy(heading.headingAccuracy),
            values: [
                heading.magneticHeading,
                heading.trueHeading
            ]
        )
    }

    private func intValue(_ value: Any?, defaultValue: Int, minValue: Int) -> Int {
        let parsed: Int
        switch value {
        case let n as NSNumber:
            parsed = n.intValue
        case let s as String:
            parsed = Int(s) ?? defaultValue
        default:
            parsed = defaultValue
        }
        return max(parsed, minValue)
    }

    func locationAuthorizationStatusValue() -> CLAuthorizationStatus {
        if #available(iOS 14.0, *) {
            return locationManager.authorizationStatus
        }
        return CLLocationManager.authorizationStatus()
    }

    private func motionTimestampToMonotonicNs(_ timestamp: TimeInterval) -> UInt64 {
        guard timestamp > 0 else { return currentMonotonicNs() }
        return UInt64(timestamp * 1_000_000_000.0)
    }

    private func locationToMonotonicNs(_ location: CLLocation) -> UInt64 {
        let nowMonoNs = currentMonotonicNs()
        let nowUnixMs = Int64(Date().timeIntervalSince1970 * 1000.0)
        let locationUnixMs = Int64(location.timestamp.timeIntervalSince1970 * 1000.0)
        let deltaMs = max(0, nowUnixMs - locationUnixMs)
        let deltaNs = UInt64(deltaMs) * 1_000_000
        return nowMonoNs > deltaNs ? nowMonoNs - deltaNs : 0
    }

    private func currentMonotonicNs() -> UInt64 {
        UInt64(ProcessInfo.processInfo.systemUptime * 1_000_000_000.0)
    }

    private func monotonicToUnixMs(_ monotonicNs: UInt64) -> Int64 {
        let nowMonoNs = currentMonotonicNs()
        let nowUnixMs = Int64(Date().timeIntervalSince1970 * 1000.0)
        let deltaMs = monotonicNs <= nowMonoNs ? Int64((nowMonoNs - monotonicNs) / 1_000_000) : 0
        return nowUnixMs - deltaMs
    }

    private func mapMotionConfidence(_ confidence: CMMotionActivityConfidence) -> String {
        switch confidence {
        case .low: return "low"
        case .medium: return "medium"
        case .high: return "high"
        @unknown default: return "medium"
        }
    }

    private func mapLocationAccuracy(_ horizontalAccuracy: CLLocationAccuracy) -> String {
        guard horizontalAccuracy >= 0 else { return "unreliable" }
        if horizontalAccuracy <= 10 { return "high" }
        if horizontalAccuracy <= 50 { return "medium" }
        return "low"
    }

    private func mapHeadingAccuracy(_ headingAccuracy: CLLocationDirection) -> String {
        guard headingAccuracy >= 0 else { return "unreliable" }
        if headingAccuracy <= 5 { return "high" }
        if headingAccuracy <= 20 { return "medium" }
        return "low"
    }

    private func toJsonString(_ object: Any, fallback: String) -> String {
        guard JSONSerialization.isValidJSONObject(object),
              let data = try? JSONSerialization.data(withJSONObject: object),
              let json = String(data: data, encoding: .utf8) else {
            return fallback
        }
        return json
    }

    private func runOnMainSync(_ block: () -> Void) {
        if Thread.isMainThread {
            block()
        } else {
            DispatchQueue.main.sync(execute: block)
        }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard currentConfig().enabled.contains("gps") else { return }
        for location in locations {
            appendLocationFrame(location)
        }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateHeading newHeading: CLHeading) {
        guard currentConfig().enabled.contains("heading") else { return }
        appendHeadingFrame(newHeading)
    }
}

private struct IOSBleScanConfig {
    var serviceUUIDs: [CBUUID] = []
    var allowDuplicates: Bool = false
    var scanMode: String?
    var frameFlushMs: Int = 500
}

private final class IOSBleCollector: NSObject, CBCentralManagerDelegate {
    private let lock = NSLock()
    private var resultBuffer: [[String: Any]] = []
    private var running = false
    private var activeSessionId: String?
    private var seq: UInt64 = 0
    private var config = IOSBleScanConfig()
    private let maxBufferedResults = 4096

    private let callbackQueue = DispatchQueue(label: "com.blinc.ble.ios.callbacks")
    private lazy var centralManager: CBCentralManager = {
        CBCentralManager(delegate: self, queue: callbackQueue)
    }()

    func configure(configJson: String) -> Bool {
        guard let data = configJson.data(using: .utf8),
              let root = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
            return false
        }

        var serviceUUIDs: [CBUUID] = []
        if let rawUuids = root["service_uuids"] as? [Any] {
            for item in rawUuids {
                guard let text = item as? String else { continue }
                let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
                if !trimmed.isEmpty {
                    serviceUUIDs.append(CBUUID(string: trimmed))
                }
            }
        }

        let allowDuplicates = (root["allow_duplicates"] as? Bool) ?? false
        let scanMode = (root["scan_mode"] as? String)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let frameFlushMs: Int
        switch root["frame_flush_ms"] {
        case let number as NSNumber:
            frameFlushMs = max(number.intValue, 0)
        case let string as String:
            frameFlushMs = max(Int(string) ?? 500, 0)
        default:
            frameFlushMs = 500
        }

        lock.lock()
        config = IOSBleScanConfig(
            serviceUUIDs: serviceUUIDs,
            allowDuplicates: allowDuplicates,
            scanMode: scanMode?.isEmpty == true ? nil : scanMode,
            frameFlushMs: frameFlushMs
        )
        lock.unlock()
        return true
    }

    func hasBluetoothPermission() -> Bool {
        if #available(iOS 13.0, *) {
            return CBManager.authorization == .allowedAlways
        }
        return true
    }

    func requestBluetoothPermission() -> Bool {
        if hasBluetoothPermission() {
            return true
        }
        _ = centralManager.state
        Thread.sleep(forTimeInterval: 0.05)
        return hasBluetoothPermission()
    }

    func start(sessionId: String) -> Bool {
        let normalizedId = sessionId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedId.isEmpty else { return false }
        guard hasBluetoothPermission() else { return false }
        guard centralManager.state == .poweredOn else { return false }

        lock.lock()
        if running {
            centralManager.stopScan()
        }
        running = true
        activeSessionId = normalizedId
        seq = 0
        resultBuffer.removeAll(keepingCapacity: true)
        let currentConfig = config
        lock.unlock()

        let serviceFilter = currentConfig.serviceUUIDs.isEmpty ? nil : currentConfig.serviceUUIDs
        centralManager.scanForPeripherals(
            withServices: serviceFilter,
            options: [CBCentralManagerScanOptionAllowDuplicatesKey: currentConfig.allowDuplicates]
        )
        return true
    }

    func stop(sessionId: String) -> Bool {
        let normalizedId = sessionId.trimmingCharacters(in: .whitespacesAndNewlines)

        lock.lock()
        if !running {
            lock.unlock()
            return true
        }
        if !normalizedId.isEmpty, normalizedId != activeSessionId {
            lock.unlock()
            return false
        }
        running = false
        activeSessionId = nil
        lock.unlock()

        centralManager.stopScan()
        return true
    }

    func statusJson() -> String {
        lock.lock()
        let payload: [String: Any] = [
            "running": running,
            "buffered_results": resultBuffer.count,
            "active_session_id": activeSessionId ?? NSNull()
        ]
        lock.unlock()
        return toJsonString(payload, fallback: #"{"running":false,"buffered_results":0,"active_session_id":null}"#)
    }

    func drainResults(maxResults: Int) -> String {
        let count = max(1, min(maxResults, 2048))
        lock.lock()
        let drainedCount = min(count, resultBuffer.count)
        let drained = Array(resultBuffer.prefix(drainedCount))
        if drainedCount > 0 {
            resultBuffer.removeFirst(drainedCount)
        }
        lock.unlock()
        return toJsonString(drained, fallback: "[]")
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {}

    func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String: Any],
        rssi RSSI: NSNumber
    ) {
        let monotonicNs = currentMonotonicNs()
        let unixTimeMs = monotonicToUnixMs(monotonicNs)
        let name = (advertisementData[CBAdvertisementDataLocalNameKey] as? String) ?? peripheral.name
        let serviceUUIDs = (advertisementData[CBAdvertisementDataServiceUUIDsKey] as? [CBUUID] ?? [])
            .map { $0.uuidString }
            .sorted()

        var txPower: Int?
        if let tx = advertisementData[CBAdvertisementDataTxPowerLevelKey] as? NSNumber {
            txPower = tx.intValue
        }

        var isConnectable: Bool?
        if let connectable = advertisementData[CBAdvertisementDataIsConnectable] as? NSNumber {
            isConnectable = connectable.boolValue
        }

        var manufacturerData: String?
        if let data = advertisementData[CBAdvertisementDataManufacturerDataKey] as? Data {
            manufacturerData = data.base64EncodedString()
        }

        var serviceData: String?
        if let map = advertisementData[CBAdvertisementDataServiceDataKey] as? [CBUUID: Data],
           let first = map.first {
            serviceData = "\(first.key.uuidString):\(first.value.base64EncodedString())"
        }

        lock.lock()
        defer { lock.unlock() }
        guard running else { return }

        seq &+= 1
        var payload: [String: Any] = [
            "seq": NSNumber(value: seq),
            "address": peripheral.identifier.uuidString,
            "rssi": RSSI.intValue,
            "service_uuids": serviceUUIDs,
            "time_monotonic_ns": NSNumber(value: monotonicNs),
            "time_unix_ms": NSNumber(value: unixTimeMs)
        ]
        payload["name"] = name ?? NSNull()
        payload["tx_power"] = txPower.map { NSNumber(value: $0) } ?? NSNull()
        payload["is_connectable"] = isConnectable.map { NSNumber(value: $0) } ?? NSNull()
        payload["manufacturer_data"] = manufacturerData ?? NSNull()
        payload["service_data"] = serviceData ?? NSNull()

        if resultBuffer.count >= maxBufferedResults {
            resultBuffer.removeFirst()
        }
        resultBuffer.append(payload)
    }

    private func currentMonotonicNs() -> UInt64 {
        UInt64(ProcessInfo.processInfo.systemUptime * 1_000_000_000.0)
    }

    private func monotonicToUnixMs(_ monotonicNs: UInt64) -> Int64 {
        let nowMonoNs = currentMonotonicNs()
        let nowUnixMs = Int64(Date().timeIntervalSince1970 * 1000.0)
        let deltaMs = monotonicNs <= nowMonoNs ? Int64((nowMonoNs - monotonicNs) / 1_000_000) : 0
        return nowUnixMs - deltaMs
    }

    private func toJsonString(_ object: Any, fallback: String) -> String {
        guard JSONSerialization.isValidJSONObject(object),
              let data = try? JSONSerialization.data(withJSONObject: object),
              let json = String(data: data, encoding: .utf8) else {
            return fallback
        }
        return json
    }
}

// MARK: - C FFI Entry Point

/// C function called by Rust to execute native handlers
/// Returns a malloc'd string that Rust must free with blinc_free_string
@_cdecl("blinc_ios_native_call")
public func blinc_ios_native_call(
    ns: UnsafePointer<CChar>?,
    name: UnsafePointer<CChar>?,
    argsJson: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard let ns, let name, let argsJson else {
        return strdup(#"{"success":false,"errorType":"PlatformError","errorMessage":"null native bridge input"}"#)
    }

    let namespace = String(cString: ns)
    let funcName = String(cString: name)
    let args = String(cString: argsJson)

    let result = BlincNativeBridge.shared.callNative(
        namespace: namespace,
        name: funcName,
        argsJson: args
    )

    return strdup(result)
}
