declare module 'save-svg-as-png' {
  export interface SaveSVGOptions {
    scale?: number;
    encoderOptions?: number;
    backgroundColor?: string;
    width?: number;
    height?: number;
    left?: number;
    top?: number;
    excludeCss?: boolean;
    includeCanvasContent?: boolean;
  }

  export function saveSvgAsPng(
    svg: Element | string,
    filename: string,
    options?: SaveSVGOptions
  ): Promise<void>;

  export function saveSvg(
    svg: Element | string,
    name: string,
    options?: SaveSVGOptions
  ): Promise<string>;

  export function prepareSvg(
    svg: Element | string,
    options?: SaveSVGOptions
  ): Promise<HTMLCanvasElement>;

  export function svgAsDataUri(svg: Element | string, options?: SaveSVGOptions): Promise<string>;

  export function svgAsPngUri(svg: Element | string, options?: SaveSVGOptions): Promise<string>;

  export function saveSvgAsPng(
    svg: Element | string,
    filename: string,
    options?: SaveSVGOptions
  ): Promise<void>;

  export default saveSvgAsPng;
}
