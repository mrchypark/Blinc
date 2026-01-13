import UIKit
import MetalKit

class BlincViewController: UIViewController {

    private var displayLink: CADisplayLink?
    private var metalLayer: CAMetalLayer?
    private var renderContext: OpaquePointer?

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black

        setupMetal()
        setupRenderContext()
        setupDisplayLink()
    }

    private func setupMetal() {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not supported on this device")
        }

        let layer = CAMetalLayer()
        layer.device = device
        layer.pixelFormat = .bgra8Unorm
        layer.framebufferOnly = true
        layer.frame = view.layer.bounds
        layer.contentsScale = UIScreen.main.scale

        view.layer.addSublayer(layer)
        metalLayer = layer
    }

    private func setupRenderContext() {
        let scale = UIScreen.main.scale
        let width = UInt32(view.bounds.width * scale)
        let height = UInt32(view.bounds.height * scale)

        renderContext = blinc_create_context(width, height, Double(scale))
    }

    private func setupDisplayLink() {
        displayLink = CADisplayLink(target: self, selector: #selector(render))
        displayLink?.add(to: .main, forMode: .common)
    }

    @objc private func render() {
        guard let ctx = renderContext else { return }

        if blinc_needs_render(ctx) {
            _ = blinc_tick_animations(ctx)
            blinc_build_frame(ctx)
            // Actual Metal rendering would happen here
        }
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        metalLayer?.frame = view.layer.bounds

        if let ctx = renderContext {
            let scale = UIScreen.main.scale
            let width = UInt32(view.bounds.width * scale)
            let height = UInt32(view.bounds.height * scale)
            blinc_update_size(ctx, width, height, Double(scale))
        }
    }

    // MARK: - Touch Handling

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }
        for touch in touches {
            let point = touch.location(in: view)
            let touchId = UInt64(touch.hash)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 0)
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }
        for touch in touches {
            let point = touch.location(in: view)
            let touchId = UInt64(touch.hash)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 1)
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }
        for touch in touches {
            let point = touch.location(in: view)
            let touchId = UInt64(touch.hash)
            blinc_handle_touch(ctx, touchId, Float(point.x), Float(point.y), 2)
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let ctx = renderContext else { return }
        for touch in touches {
            let touchId = UInt64(touch.hash)
            blinc_handle_touch(ctx, touchId, 0, 0, 3)
        }
    }

    // MARK: - Lifecycle

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        if let ctx = renderContext {
            blinc_set_focused(ctx, true)
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        if let ctx = renderContext {
            blinc_set_focused(ctx, false)
        }
    }

    deinit {
        displayLink?.invalidate()
        if let ctx = renderContext {
            blinc_destroy_context(ctx)
        }
    }
}
