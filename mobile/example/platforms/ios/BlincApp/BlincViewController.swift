import UIKit
import MetalKit
import os.log

private let log = OSLog(subsystem: "com.blinc.example", category: "BlincViewController")

/// Main view controller for Blinc iOS applications
///
/// This controller manages the Metal rendering surface, display link for
/// frame timing, and touch event routing to the Blinc framework.
///
/// Usage:
/// 1. Set as root view controller in AppDelegate
/// 2. Call `registerUIBuilder(_:)` to set up your UI
/// 3. The controller handles rendering and events automatically
class BlincViewController: UIViewController {

    // MARK: - Properties

    /// The Metal view for rendering
    private var metalView: BlincMetalView!

    /// CADisplayLink for frame timing
    private var displayLink: CADisplayLink?

    /// Blinc render context (manages UI state)
    private var renderContext: OpaquePointer?

    /// Blinc GPU renderer (manages Metal rendering)
    private var gpuRenderer: OpaquePointer?

    /// Whether the view is currently visible
    private var isVisible = false

    /// Track touches by their hash for multi-touch support
    private var touchIds: [ObjectIdentifier: UInt64] = [:]
    private var nextTouchId: UInt64 = 1

    /// Sensor debug panel shown on top of the app for runtime verification.
    private var sensorDebugCardView: UIView?
    private var sensorDebugStatusLabel: UILabel?
    private var sensorDebugBodyLabel: UILabel?
    private var sensorDebugTimer: Timer?
    private var sensorPollBatch: UInt64 = 0

    // MARK: - Lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        // Create Metal view
        metalView = BlincMetalView(frame: view.bounds)
        metalView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        view.addSubview(metalView)

        // Initialize Blinc context
        initializeBlinc()

        // Set up display link
        setupDisplayLink()
        setupSensorDebugOverlay()
        refreshSensorDebugOverlay()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        isVisible = true
        displayLink?.isPaused = false

        if let ctx = renderContext {
            blinc_set_focused(ctx, true)
        }
        startSensorDebugTimer()
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        isVisible = false
        displayLink?.isPaused = true

        if let ctx = renderContext {
            blinc_set_focused(ctx, false)
        }
        stopSensorDebugTimer()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        // Update Blinc with new size
        let scale = UIScreen.main.scale
        let width = UInt32(view.bounds.width * scale)
        let height = UInt32(view.bounds.height * scale)

        if let ctx = renderContext {
            blinc_update_size(ctx, width, height, Double(scale))
        }

        if let gpu = gpuRenderer {
            blinc_gpu_resize(gpu, width, height)
        }
    }

    deinit {
        // Stop display link
        displayLink?.invalidate()
        displayLink = nil
        stopSensorDebugTimer()

        // Clean up Blinc resources
        if let gpu = gpuRenderer {
            blinc_destroy_gpu(gpu)
        }
        if let ctx = renderContext {
            blinc_destroy_context(ctx)
        }
    }

    // MARK: - Initialization

    private func initializeBlinc() {
        let scale = UIScreen.main.scale
        let width = UInt32(view.bounds.width * scale)
        let height = UInt32(view.bounds.height * scale)

        os_log(.info, log: log, "Starting initialization %dx%d @ %.1fx", width, height, scale)

        // Initialize the Rust app (registers UI builder)
        os_log(.info, log: log, "Calling ios_app_init()")
        ios_app_init()

        // Create render context
        os_log(.info, log: log, "Calling blinc_create_context()")
        guard let ctx = blinc_create_context(width, height, Double(scale)) else {
            os_log(.error, log: log, "Failed to create Blinc render context")
            return
        }
        renderContext = ctx
        os_log(.info, log: log, "Render context created")

        // Initialize GPU with Metal layer
        let metalLayer = metalView.metalLayer
        os_log(.info, log: log, "Metal layer device: %{public}@", String(describing: metalLayer.device))

        let layerPtr = Unmanaged.passUnretained(metalLayer).toOpaque()
        os_log(.info, log: log, "Calling blinc_init_gpu()")

        guard let gpu = blinc_init_gpu(ctx, layerPtr, width, height) else {
            os_log(.error, log: log, "Failed to initialize Blinc GPU renderer")
            return
        }
        gpuRenderer = gpu

        // Load bundled fonts from app bundle
        loadBundledFonts(gpu: gpu)

        os_log(.info, log: log, "Blinc fully initialized: %dx%d @ %.1fx", width, height, scale)
    }

    /// Load bundled fonts from the app bundle
    private func loadBundledFonts(gpu: OpaquePointer) {
        // Get the bundle path for fonts
        let fontNames = ["Arial.ttf"]

        for fontName in fontNames {
            if let fontPath = Bundle.main.path(forResource: fontName.replacingOccurrences(of: ".ttf", with: ""),
                                                ofType: "ttf") {
                os_log(.info, log: log, "Loading bundled font: %{public}@", fontPath)
                let loaded = blinc_load_bundled_font(gpu, fontPath)
                os_log(.info, log: log, "Loaded %d font faces from %{public}@", loaded, fontName)
            } else {
                os_log(.fault, log: log, "Bundled font not found: %{public}@", fontName)
            }
        }
    }

    private func setupDisplayLink() {
        displayLink = CADisplayLink(target: self, selector: #selector(displayLinkFired))

        // Prefer 60fps, but allow system to throttle
        if #available(iOS 15.0, *) {
            displayLink?.preferredFrameRateRange = CAFrameRateRange(minimum: 30, maximum: 120, preferred: 60)
        } else {
            displayLink?.preferredFramesPerSecond = 60
        }

        displayLink?.add(to: .main, forMode: .common)
    }

    // MARK: - Sensor Debug Overlay

    private func setupSensorDebugOverlay() {
        let card = UIView()
        card.translatesAutoresizingMaskIntoConstraints = false
        card.backgroundColor = UIColor(red: 0.18, green: 0.18, blue: 0.23, alpha: 0.94)
        card.layer.cornerRadius = 12
        card.layer.borderWidth = 1
        card.layer.borderColor = UIColor.white.withAlphaComponent(0.08).cgColor
        card.layer.masksToBounds = true
        card.isUserInteractionEnabled = false

        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.text = "Sensor Debug"
        titleLabel.textColor = .white
        titleLabel.font = UIFont.systemFont(ofSize: 14, weight: .semibold)

        let statusLabel = UILabel()
        statusLabel.translatesAutoresizingMaskIntoConstraints = false
        statusLabel.text = "IDLE"
        statusLabel.textAlignment = .center
        statusLabel.textColor = UIColor(red: 0.68, green: 0.92, blue: 1.0, alpha: 1.0)
        statusLabel.font = UIFont.monospacedSystemFont(ofSize: 10, weight: .bold)
        statusLabel.backgroundColor = UIColor(red: 0.13, green: 0.24, blue: 0.33, alpha: 1.0)
        statusLabel.layer.cornerRadius = 7
        statusLabel.layer.masksToBounds = true

        let bodyLabel = UILabel()
        bodyLabel.translatesAutoresizingMaskIntoConstraints = false
        bodyLabel.numberOfLines = 0
        bodyLabel.textColor = UIColor(red: 0.90, green: 0.94, blue: 1.0, alpha: 1.0)
        bodyLabel.font = UIFont.monospacedSystemFont(ofSize: 11, weight: .regular)
        bodyLabel.text = "initializing..."

        card.addSubview(titleLabel)
        card.addSubview(statusLabel)
        card.addSubview(bodyLabel)
        view.addSubview(card)
        view.bringSubviewToFront(card)

        sensorDebugCardView = card
        sensorDebugStatusLabel = statusLabel
        sensorDebugBodyLabel = bodyLabel

        NSLayoutConstraint.activate([
            card.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 8),
            card.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -8),
            card.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 8),
            card.heightAnchor.constraint(equalToConstant: 170),

            titleLabel.leadingAnchor.constraint(equalTo: card.leadingAnchor, constant: 12),
            titleLabel.topAnchor.constraint(equalTo: card.topAnchor, constant: 10),

            statusLabel.trailingAnchor.constraint(equalTo: card.trailingAnchor, constant: -12),
            statusLabel.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor),
            statusLabel.widthAnchor.constraint(equalToConstant: 62),
            statusLabel.heightAnchor.constraint(equalToConstant: 18),

            bodyLabel.leadingAnchor.constraint(equalTo: card.leadingAnchor, constant: 12),
            bodyLabel.trailingAnchor.constraint(equalTo: card.trailingAnchor, constant: -12),
            bodyLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            bodyLabel.bottomAnchor.constraint(lessThanOrEqualTo: card.bottomAnchor, constant: -10),
        ])
    }

    private func startSensorDebugTimer() {
        stopSensorDebugTimer()
        sensorDebugTimer = Timer.scheduledTimer(
            timeInterval: 0.6,
            target: self,
            selector: #selector(sensorDebugTimerFired),
            userInfo: nil,
            repeats: true
        )
        if let timer = sensorDebugTimer {
            RunLoop.main.add(timer, forMode: .common)
        }
    }

    private func stopSensorDebugTimer() {
        sensorDebugTimer?.invalidate()
        sensorDebugTimer = nil
    }

    @objc private func sensorDebugTimerFired() {
        refreshSensorDebugOverlay()
    }

    private func refreshSensorDebugOverlay() {
        guard let bodyLabel = sensorDebugBodyLabel else { return }

        let statusObject = sensorStatusObject()
        let frames = sensorPeekFrames(maxFrames: 32)
        sensorPollBatch += 1

        var counts: [String: Int] = [:]
        for frame in frames {
            let kind = frame["sensor"] as? String ?? "unknown"
            counts[kind, default: 0] += 1
        }

        let sortedKinds = counts.keys.sorted()
        let kindsSummary = sortedKinds
            .map { "\($0)=\(counts[$0] ?? 0)" }
            .joined(separator: ", ")

        let sampleText: String
        if let frame = frames.last {
            let sensor = frame["sensor"] as? String ?? "unknown"
            let values = (frame["values"] as? [NSNumber]) ?? []
            let compact = values.prefix(4).map { String(format: "%.3f", $0.doubleValue) }.joined(separator: ", ")
            sampleText = "\(sensor): [\(compact)]"
        } else {
            sampleText = "no frames yet"
        }

        let buffered = statusObject?["buffered_frames"] as? Int ?? 0
        let running = statusObject?["running"] as? Bool ?? false
        let sessionId = statusObject?["active_session_id"] as? String ?? "-"
        let supported = sensorSupportedKinds().joined(separator: ", ")

        sensorDebugStatusLabel?.text = running ? "RUNNING" : "IDLE"
        sensorDebugStatusLabel?.textColor = running
            ? UIColor(red: 0.77, green: 1.0, blue: 0.84, alpha: 1.0)
            : UIColor(red: 0.68, green: 0.92, blue: 1.0, alpha: 1.0)
        sensorDebugStatusLabel?.backgroundColor = running
            ? UIColor(red: 0.14, green: 0.38, blue: 0.22, alpha: 1.0)
            : UIColor(red: 0.13, green: 0.24, blue: 0.33, alpha: 1.0)

        bodyLabel.text = """
        session: \(sessionId)  buffered: \(buffered)
        poll: \(sensorPollBatch)  peeked: \(frames.count)
        supported: [\(supported)]
        kinds: [\(kindsSummary)]
        sample: \(sampleText)
        """
    }

    private func callBridgeValue(namespace: String, name: String, args: [Any] = []) -> Any? {
        guard JSONSerialization.isValidJSONObject(args),
              let argsData = try? JSONSerialization.data(withJSONObject: args),
              let argsJson = String(data: argsData, encoding: .utf8) else {
            return nil
        }

        let responseJson = BlincNativeBridge.shared.callNative(
            namespace: namespace,
            name: name,
            argsJson: argsJson
        )

        guard let responseData = responseJson.data(using: .utf8),
              let response = try? JSONSerialization.jsonObject(with: responseData) as? [String: Any],
              let success = response["success"] as? Bool else {
            return nil
        }
        if !success {
            return nil
        }
        return response["value"]
    }

    private func sensorStatusObject() -> [String: Any]? {
        guard let statusJson = callBridgeValue(namespace: "sensor", name: "status") as? String,
              let data = statusJson.data(using: .utf8),
              let status = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }
        return status
    }

    private func sensorPeekFrames(maxFrames: Int) -> [[String: Any]] {
        guard let framesJson = callBridgeValue(namespace: "sensor", name: "peek_frames", args: [maxFrames]) as? String,
              let data = framesJson.data(using: .utf8),
              let frames = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
            return []
        }
        return frames
    }

    private func sensorSupportedKinds() -> [String] {
        guard let kindsJson = callBridgeValue(namespace: "sensor", name: "supported_kinds") as? String,
              let data = kindsJson.data(using: .utf8),
              let kinds = try? JSONSerialization.jsonObject(with: data) as? [String] else {
            return []
        }
        return kinds
    }

    // MARK: - Rendering

    private var frameCount = 0

    @objc private func displayLinkFired() {
        guard isVisible,
              let ctx = renderContext,
              let gpu = gpuRenderer else {
            if frameCount == 0 {
                os_log(.error, log: log, "displayLinkFired - missing context or gpu (isVisible: %d, ctx: %d, gpu: %d)",
                       isVisible ? 1 : 0, renderContext != nil ? 1 : 0, gpuRenderer != nil ? 1 : 0)
            }
            return
        }

        // Log first few frames
        if frameCount < 3 {
            os_log(.debug, log: log, "Frame %d - checking if render needed", frameCount)
        }

        // Check if we need to render
        let needsRender = blinc_needs_render(ctx)
        if frameCount < 3 {
            os_log(.debug, log: log, "Frame %d - needs_render: %d", frameCount, needsRender ? 1 : 0)
        }

        if !needsRender && frameCount >= 3 {
            return
        }

        // Build UI (this ticks animations and calls the UI builder)
        if frameCount < 3 {
            os_log(.debug, log: log, "Frame %d - calling blinc_build_frame", frameCount)
        }
        blinc_build_frame(ctx)

        // Render to Metal
        if frameCount < 3 {
            os_log(.debug, log: log, "Frame %d - calling blinc_render_frame", frameCount)
        }
        let result = blinc_render_frame(gpu)
        if frameCount < 3 {
            os_log(.info, log: log, "Frame %d - render result: %d", frameCount, result ? 1 : 0)
        }

        frameCount += 1
    }

    // MARK: - Touch Handling

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        os_log(.info, log: log, "touchesBegan: %d touches, renderContext=%{public}@",
               touches.count, renderContext != nil ? "valid" : "nil")

        guard let ctx = renderContext else {
            os_log(.error, log: log, "touchesBegan: renderContext is nil!")
            return
        }

        for touch in touches {
            let point = touch.location(in: view)
            let touchId = getTouchId(for: touch)
            os_log(.info, log: log, "touchesBegan: calling blinc_handle_touch at (%.1f, %.1f)", point.x, point.y)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 0) // 0 = began
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }

        for touch in touches {
            let point = touch.location(in: view)
            let touchId = getTouchId(for: touch)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 1) // 1 = moved
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        os_log(.info, log: log, "touchesEnded: %d touches", touches.count)

        guard let ctx = renderContext else {
            os_log(.error, log: log, "touchesEnded: renderContext is nil!")
            return
        }

        for touch in touches {
            let point = touch.location(in: view)
            let touchId = getTouchId(for: touch)
            os_log(.info, log: log, "touchesEnded: calling blinc_handle_touch at (%.1f, %.1f)", point.x, point.y)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 2) // 2 = ended
            removeTouchId(for: touch)
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }

        for touch in touches {
            let point = touch.location(in: view)
            let touchId = getTouchId(for: touch)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 3) // 3 = cancelled
            removeTouchId(for: touch)
        }
    }

    // MARK: - Touch ID Management

    private func getTouchId(for touch: UITouch) -> UInt64 {
        let identifier = ObjectIdentifier(touch)
        if let existingId = touchIds[identifier] {
            return existingId
        }
        let newId = nextTouchId
        nextTouchId += 1
        touchIds[identifier] = newId
        return newId
    }

    private func removeTouchId(for touch: UITouch) {
        let identifier = ObjectIdentifier(touch)
        touchIds.removeValue(forKey: identifier)
    }

    // MARK: - Status Bar

    override var prefersStatusBarHidden: Bool {
        return true
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    // MARK: - Safe Area

    override var preferredScreenEdgesDeferringSystemGestures: UIRectEdge {
        return .all
    }
}

// MARK: - UI Builder Registration

/// Global UI builder function pointer for FFI
private var globalUIBuilder: UIBuilderFn?

/// Register a UI builder function for the application
///
/// This function should be called from your app's Rust code via FFI
/// before the view controller is created.
///
/// Example Rust:
/// ```rust
/// #[no_mangle]
/// pub extern "C" fn my_ui_builder(ctx: *mut WindowedContext) {
///     // Build UI here
/// }
///
/// fn main() {
///     blinc_set_ui_builder(my_ui_builder);
/// }
/// ```
func registerUIBuilder(_ builder: UIBuilderFn) {
    blinc_set_ui_builder(builder)
}
