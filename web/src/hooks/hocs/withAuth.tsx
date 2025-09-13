import React, { ComponentType, createContext, useContext, ReactNode } from 'react';
import { Box, CircularProgress, Alert, Button } from '@mui/material';

/**
 * User object interface
 */
export interface User {
  id: string;
  name: string;
  email: string;
  roles?: string[];
  permissions?: string[];
}

/**
 * Authentication state interface
 */
export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

/**
 * Authentication context interface
 */
export interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
  hasRole: (role: string) => boolean;
  hasPermission: (permission: string) => boolean;
}

/**
 * Default authentication context
 */
const AuthContext = createContext<AuthContextValue | null>(null);

/**
 * Hook to access authentication context
 */
export const useAuth = (): AuthContextValue => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

/**
 * Authentication provider props
 */
export interface AuthProviderProps {
  children: ReactNode;
}

/**
 * Higher-Order Component for authentication
 *
 * Wraps a component requiring authentication and shows appropriate UI
 * when user is not authenticated.
 *
 * @param WrappedComponent - Component to protect
 * @param options - Authentication options
 * @returns Protected component
 *
 * @example
 * ```tsx
 * const ProtectedDashboard = withAuth(Dashboard, {
 *   requiredRoles: ['admin'],
 *   fallbackComponent: LoginComponent
 * });
 * ```
 */
export function withAuth<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: WithAuthOptions = {}
): ComponentType<P> {
  const {
    requiredRoles = [],
    requiredPermissions = [],
    fallbackComponent: FallbackComponent,
    loadingComponent: LoadingComponent,
    redirectTo,
    showLoginButton = true,
  } = options;

  const AuthenticatedComponent: React.FC<P> = (props) => {
    const auth = useAuth();

    // Check authentication
    if (auth.isLoading) {
      if (LoadingComponent) {
        return <LoadingComponent />;
      }
      return (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
          <CircularProgress />
        </Box>
      );
    }

    // Check if user is authenticated
    if (!auth.isAuthenticated || !auth.user) {
      if (FallbackComponent) {
        return <FallbackComponent {...props} />;
      }

      if (redirectTo) {
        // Handle redirect
        window.location.href = redirectTo;
        return null;
      }

      return <DefaultAuthFallback showLoginButton={showLoginButton} />;
    }

    // Check required roles
    if (requiredRoles.length > 0) {
      const hasRequiredRole = requiredRoles.some((role) => auth.hasRole(role));
      if (!hasRequiredRole) {
        return (
          <Alert severity="error" sx={{ m: 2 }}>
            You don't have the required permissions to access this resource.
          </Alert>
        );
      }
    }

    // Check required permissions
    if (requiredPermissions.length > 0) {
      const hasRequiredPermission = requiredPermissions.some((permission) =>
        auth.hasPermission(permission)
      );
      if (!hasRequiredPermission) {
        return (
          <Alert severity="error" sx={{ m: 2 }}>
            You don't have the required permissions to access this resource.
          </Alert>
        );
      }
    }

    return <WrappedComponent {...props} />;
  };

  AuthenticatedComponent.displayName = `withAuth(${WrappedComponent.displayName || WrappedComponent.name})`;

  return AuthenticatedComponent;
}

/**
 * Options for withAuth HOC
 */
export interface WithAuthOptions {
  /** Required roles for accessing the component */
  requiredRoles?: string[];
  /** Required permissions for accessing the component */
  requiredPermissions?: string[];
  /** Custom fallback component when not authenticated */
  fallbackComponent?: ComponentType<any>;
  /** Custom loading component */
  loadingComponent?: ComponentType<any>;
  /** Redirect URL when not authenticated */
  redirectTo?: string;
  /** Whether to show login button in default fallback */
  showLoginButton?: boolean;
}

/**
 * Default authentication fallback component
 */
const DefaultAuthFallback: React.FC<{ showLoginButton?: boolean }> = ({
  showLoginButton = true,
}) => (
  <Box
    display="flex"
    flexDirection="column"
    alignItems="center"
    justifyContent="center"
    minHeight="300px"
    p={3}
  >
    <Alert severity="warning" sx={{ mb: 2, maxWidth: 400 }}>
      You need to be logged in to access this resource.
    </Alert>
    {showLoginButton && (
      <Button variant="contained" color="primary" onClick={() => (window.location.href = '/login')}>
        Go to Login
      </Button>
    )}
  </Box>
);

/**
 * Role-based authentication HOC
 */
export function withRoles<P extends object>(
  WrappedComponent: ComponentType<P>,
  roles: string[],
  options: Omit<WithAuthOptions, 'requiredRoles'> = {}
): ComponentType<P> {
  return withAuth(WrappedComponent, { ...options, requiredRoles: roles });
}

/**
 * Permission-based authentication HOC
 */
export function withPermissions<P extends object>(
  WrappedComponent: ComponentType<P>,
  permissions: string[],
  options: Omit<WithAuthOptions, 'requiredPermissions'> = {}
): ComponentType<P> {
  return withAuth(WrappedComponent, { ...options, requiredPermissions: permissions });
}

/**
 * Conditional authentication HOC - only applies auth if condition is met
 */
export function withConditionalAuth<P extends object>(
  WrappedComponent: ComponentType<P>,
  conditionFn: (props: P) => boolean,
  options: WithAuthOptions = {}
): ComponentType<P> {
  const ConditionalComponent: React.FC<P> = (props) => {
    const shouldAuthenticate = conditionFn(props);

    if (shouldAuthenticate) {
      const AuthenticatedComponent = withAuth(WrappedComponent, options);
      return <AuthenticatedComponent {...props} />;
    }

    return <WrappedComponent {...props} />;
  };

  ConditionalComponent.displayName = `withConditionalAuth(${WrappedComponent.displayName || WrappedComponent.name})`;

  return ConditionalComponent;
}

/**
 * Authentication provider component
 *
 * This is a placeholder implementation. Replace with actual authentication logic
 * (e.g., JWT tokens, OAuth, session management, etc.)
 */
export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  // Placeholder implementation - replace with actual auth logic
  const [authState, setAuthState] = React.useState<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
  });

  const login = async (email: string, password: string): Promise<void> => {
    setAuthState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // Placeholder login logic - replace with actual API call
      if (email && password) {
        const mockUser: User = {
          id: '1',
          name: 'John Doe',
          email,
          roles: ['user'],
          permissions: ['read', 'write'],
        };

        setAuthState({
          user: mockUser,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });

        localStorage.setItem('auth_user', JSON.stringify(mockUser));
      } else {
        throw new Error('Invalid credentials');
      }
    } catch (error) {
      setAuthState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
      throw error;
    }
  };

  const logout = async (): Promise<void> => {
    setAuthState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
    localStorage.removeItem('auth_user');
  };

  const checkAuth = async (): Promise<void> => {
    setAuthState((prev) => ({ ...prev, isLoading: true }));

    try {
      // Check stored auth data - replace with actual validation
      const storedUser = localStorage.getItem('auth_user');
      if (storedUser) {
        const user: User = JSON.parse(storedUser);
        setAuthState({
          user,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
      } else {
        setAuthState((prev) => ({ ...prev, isLoading: false }));
      }
    } catch (error) {
      setAuthState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Auth check failed',
      }));
    }
  };

  const hasRole = (role: string): boolean => {
    return authState.user?.roles?.includes(role) ?? false;
  };

  const hasPermission = (permission: string): boolean => {
    return authState.user?.permissions?.includes(permission) ?? false;
  };

  const contextValue: AuthContextValue = {
    ...authState,
    login,
    logout,
    checkAuth,
    hasRole,
    hasPermission,
  };

  return <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>;
};

// Export the context for advanced usage
export { AuthContext };
