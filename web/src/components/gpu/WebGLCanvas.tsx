import React, { useRef, useEffect, useCallback, useState, useContext } from 'react';
import { Box, Typography, Alert, useTheme } from '@mui/material';

// WebGL Context interface for TypeScript
interface WebGLRenderingContextWithDeprecations extends WebGLRenderingContext {
  readonly canvas: HTMLCanvasElement;
}

// Props for the WebGL Canvas component
interface WebGLCanvasProps {
  width: number;
  height: number;
  backgroundColor?: [number, number, number, number];
  onRender?: (gl: WebGLRenderingContext, time: number) => void;
  onInit?: (gl: WebGLRenderingContext) => void;
  onResize?: (newWidth: number, newHeight: number) => void;
  enableDepth?: boolean;
  enableAlpha?: boolean;
  showFallback?: boolean;
}

// Fallback Canvas 2D context for syntax highlighting
const Canvas2DContext = React.createContext<CanvasRenderingContext2D | null>(null);

export const useCanvas2D = () => {
  return useContext(Canvas2DContext);
};

// WebGL program cache for shader reuse
class ShaderProgramCache {
  private cache = new Map<string, WebGLProgram>();

  get(
    gl: WebGLRenderingContext,
    vertexShader: string,
    fragmentShader: string
  ): WebGLProgram | null {
    const key = `${vertexShader}_${fragmentShader}`;
    const cached = this.cache.get(key);
    if (cached) return cached;

    const program = this.createProgram(gl, vertexShader, fragmentShader);
    if (program) {
      this.cache.set(key, program);
    }
    return program;
  }

  private createProgram(
    gl: WebGLRenderingContext,
    vertexSrc: string,
    fragmentSrc: string
  ): WebGLProgram | null {
    const vertexShader = this.createShader(gl, gl.VERTEX_SHADER, vertexSrc);
    const fragmentShader = this.createShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

    if (!vertexShader || !fragmentShader) return null;

    const program = gl.createProgram();
    if (!program) return null;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Shader program linking failed:', gl.getProgramInfoLog(program));
      gl.deleteProgram(program);
      return null;
    }

    return program;
  }

  private createShader(
    gl: WebGLRenderingContext,
    type: number,
    source: string
  ): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error('Shader compilation failed:', gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }
}

const shaderCache = new ShaderProgramCache();

// Basic shaders for text rendering
const vertexShaderSrc = `
  attribute vec2 a_position;
  attribute vec2 a_texCoord;
  uniform mat4 u_projection;
  varying vec2 v_texCoord;

  void main() {
    gl_Position = u_projection * vec4(a_position, 0.0, 1.0);
    v_texCoord = a_texCoord;
  }
`;

const fragmentShaderSrc = `
  precision mediump float;
  uniform vec4 u_color;
  varying vec2 v_texCoord;

  void main() {
    gl_FragColor = u_color;
  }
`;

export const WebGLCanvas: React.FC<WebGLCanvasProps> = ({
  width,
  height,
  backgroundColor = [0.0, 0.0, 0.0, 1.0],
  onRender,
  onInit,
  onResize,
  enableDepth = true,
  enableAlpha = false,
  showFallback = true,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const glRef = useRef<WebGLRenderingContext | null>(null);
  const canvas2DRef = useRef<HTMLCanvasElement>(null);
  const ctx2DRef = useRef<CanvasRenderingContext2D | null>(null);
  const animationFrameRef = useRef<number>();
  const startTimeRef = useRef<number>(Date.now());
  const [webGLSupported, setWebGLSupported] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const theme = useTheme();

  // Initialize WebGL context
  const initWebGL = useCallback((): boolean => {
    const canvas = canvasRef.current;
    if (!canvas) return false;

    try {
      const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
      if (!gl) {
        setError('WebGL not supported');
        setWebGLSupported(false);
        return false;
      }

      glRef.current = gl as WebGLRenderingContextWithDeprecations;

      // Configure WebGL
      gl.enable(enableDepth ? gl.DEPTH_TEST : gl.NONE);
      gl.enable(enableAlpha ? gl.BLEND : gl.NONE);
      if (enableAlpha) {
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      }

      gl.viewport(0, 0, width, height);

      // Clear background
      gl.clearColor(...backgroundColor);
      gl.clear(gl.COLOR_BUFFER_BIT | (enableDepth ? gl.DEPTH_BUFFER_BIT : 0));

      onInit?.(gl);
      setWebGLSupported(true);
      return true;
    } catch (err) {
      console.error('WebGL initialization failed:', err);
      setError('WebGL initialization failed');
      setWebGLSupported(false);
      return false;
    }
  }, [width, height, backgroundColor, onInit, enableDepth, enableAlpha]);

  // Initialize Canvas 2D fallback
  const initCanvas2D = useCallback((): boolean => {
    const canvas = canvas2DRef.current;
    if (!canvas) return false;

    try {
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        setError('Canvas 2D not supported');
        return false;
      }

      ctx2DRef.current = ctx;
      canvas.width = width;
      canvas.height = height;

      // Clear background
      ctx.fillStyle = `rgba(${backgroundColor[0] * 255}, ${backgroundColor[1] * 255}, ${backgroundColor[2] * 255}, ${backgroundColor[3]})`;
      ctx.fillRect(0, 0, width, height);

      setWebGLSupported(false);
      return true;
    } catch (err) {
      console.error('Canvas 2D initialization failed:', err);
      setError('Canvas 2D initialization failed');
      return false;
    }
  }, [width, height, backgroundColor]);

  // Render loop
  const render = useCallback(() => {
    const currentTime = Date.now();
    const elapsed = currentTime - startTimeRef.current;

    if (webGLSupported && glRef.current) {
      onRender?.(glRef.current, elapsed);
      glRef.current.flush();
    } else if (ctx2DRef.current) {
      // Fallback rendering logic can be added here
    }

    animationFrameRef.current = requestAnimationFrame(render);
  }, [webGLSupported, onRender]);

  // Handle canvas resize
  const handleResize = useCallback(
    (newWidth: number, newHeight: number) => {
      if (canvasRef.current) {
        canvasRef.current.width = newWidth;
        canvasRef.current.height = newHeight;
      }

      if (canvas2DRef.current) {
        canvas2DRef.current.width = newWidth;
        canvas2DRef.current.height = newHeight;
      }

      if (glRef.current) {
        glRef.current.viewport(0, 0, newWidth, newHeight);
      }

      onResize?.(newWidth, newHeight);
    },
    [onResize]
  );

  // Initialize contexts on mount/update
  useEffect(() => {
    startTimeRef.current = Date.now();

    if (!initWebGL() && showFallback) {
      initCanvas2D();
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [initWebGL, initCanvas2D, showFallback]);

  // Start render loop when ready
  useEffect(() => {
    if (webGLSupported !== null) {
      render();
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [webGLSupported, render]);

  // Handle width/height changes
  useEffect(() => {
    if (
      canvasRef.current &&
      (canvasRef.current.width !== width || canvasRef.current.height !== height)
    ) {
      handleResize(width, height);
    }
  }, [width, height, handleResize]);

  // Clear resources on unmount
  useEffect(() => {
    return () => {
      if (glRef.current) {
        // Clear WebGL resources
        const gl = glRef.current;
        const numTextureUnits = gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS);
        for (let i = 0; i < numTextureUnits; i++) {
          gl.activeTexture(gl.TEXTURE0 + i);
          gl.bindTexture(gl.TEXTURE_2D, null);
        }
      }
    };
  }, []);

  return (
    <Box sx={{ position: 'relative', width, height }}>
      {/* WebGL Canvas */}
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          display: webGLSupported ? 'block' : 'none',
          backgroundColor: theme.palette.background.paper,
        }}
      />

      {/* Fallback Canvas 2D */}
      {showFallback && (
        <canvas
          ref={canvas2DRef}
          width={width}
          height={height}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            display: webGLSupported === false ? 'block' : 'none',
            backgroundColor: theme.palette.background.default,
          }}
        />
      )}

      {/* Error message */}
      {error && (
        <Box sx={{ position: 'absolute', top: 0, left: 0, p: 2 }}>
          <Alert severity="warning" sx={{ bgcolor: theme.palette.background.paper }}>
            <Typography variant="body2">{error}. Performance may be reduced.</Typography>
          </Alert>
        </Box>
      )}

      {/* Canvas 2D Context Provider */}
      {!webGLSupported && ctx2DRef.current && (
        <Canvas2DContext.Provider value={ctx2DRef.current}>
          {/* Children can use useCanvas2D hook for fallback rendering */}
        </Canvas2DContext.Provider>
      )}
    </Box>
  );
};

export default WebGLCanvas;
