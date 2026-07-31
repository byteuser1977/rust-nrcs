/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_WS_URL: string
  readonly VITE_CHAIN_ID: string
  readonly VITE_DEBUG: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare module 'qrcode' {
  interface QRCodeToDataURLOptions {
    width?: number
    margin?: number
    color?: { dark?: string; light?: string }
    errorCorrectionLevel?: 'L' | 'M' | 'Q' | 'H'
    type?: string
    quality?: number
  }
  export function toDataURL(
    text: string,
    options?: QRCodeToDataURLOptions,
  ): Promise<string>
  export function toCanvas(
    canvas: HTMLCanvasElement,
    text: string,
    options?: QRCodeToDataURLOptions,
  ): Promise<void>
  const _default: {
    toDataURL: typeof toDataURL
    toCanvas: typeof toCanvas
  }
  export default _default
}

declare module 'big.js' {
  interface BigConstructor {
    (value: string | number | Big): Big
    new (value: string | number | Big): Big
    DP: number
    RM: number
    NE: number
    PE: number
    strict: boolean
    roundDown: 0
    roundHalfUp: 1
    roundHalfEven: 2
    roundUp: 3
  }

  interface Big {
    abs(): Big
    plus(n: string | number | Big): Big
    minus(n: string | number | Big): Big
    times(n: string | number | Big): Big
    div(n: string | number | Big): Big
    mod(n: string | number | Big): Big
    pow(n: string | number | Big): Big
    sqrt(): Big
    toNumber(): number
    toFixed(dp?: number): string
    cmp(n: string | number | Big): -1 | 0 | 1
    eq(n: string | number | Big): boolean
    gt(n: string | number | Big): boolean
    gte(n: string | number | Big): boolean
    lt(n: string | number | Big): boolean
    lte(n: string | number | Big): boolean
    toString(): string
    toJSON(): string
    valueOf(): string
  }

  const _Big: BigConstructor
  export default _Big
}
