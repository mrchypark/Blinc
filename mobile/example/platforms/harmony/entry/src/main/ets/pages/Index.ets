import blinc from 'libexample.so'

@Entry
@Component
struct Index {
  private blincContext: number = 0

  build() {
    Column() {
      XComponent({
        id: 'blinc_canvas',
        type: XComponentType.SURFACE,
        libraryname: 'example'
      })
      .width('100%')
      .height('100%')
      .onLoad((xComponentContext) => {
        console.info('[Blinc] XComponent loaded')
        // Initialize Blinc native module
        this.blincContext = blinc.init(xComponentContext)
      })
      .onDestroy(() => {
        console.info('[Blinc] XComponent destroyed')
        if (this.blincContext !== 0) {
          blinc.destroy(this.blincContext)
          this.blincContext = 0
        }
      })
    }
    .width('100%')
    .height('100%')
  }
}
