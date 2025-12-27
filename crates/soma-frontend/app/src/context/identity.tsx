"use client";

import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
} from "react";
import fetchClient from "@/lib/api-client";

/**
 * Token state stored in the identity context
 */
interface TokenState {
	accessToken: string;
	refreshToken: string;
	expiresAt: number; // Unix timestamp in milliseconds
}

interface IdentityContextValue {
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
	accessToken: string | null;
	logout: () => void;
}

const IdentityContext = createContext<IdentityContextValue | undefined>(
	undefined,
);

const TOKEN_STORAGE_KEY = "soma_identity_tokens";
const REFRESH_BUFFER_MS = 60 * 1000; // Refresh 60 seconds before expiry

/**
 * Hook to access the identity context
 */
export function useIdentity() {
	const ctx = useContext(IdentityContext);
	if (!ctx) {
		throw new Error("useIdentity must be used within <IdentityProvider>");
	}
	return ctx;
}

/**
 * Load tokens from local storage
 */
function loadTokensFromStorage(): TokenState | null {
	try {
		const stored = localStorage.getItem(TOKEN_STORAGE_KEY);
		if (stored) {
			return JSON.parse(stored) as TokenState;
		}
	} catch {
		// Ignore parse errors
	}
	return null;
}

/**
 * Save tokens to local storage
 */
function saveTokensToStorage(tokens: TokenState): void {
	localStorage.setItem(TOKEN_STORAGE_KEY, JSON.stringify(tokens));
}

/**
 * Clear tokens from local storage
 */
function clearTokensFromStorage(): void {
	localStorage.removeItem(TOKEN_STORAGE_KEY);
}

/**
 * Check if tokens are expired or about to expire
 */
function isTokenExpired(tokens: TokenState): boolean {
	return Date.now() >= tokens.expiresAt - REFRESH_BUFFER_MS;
}

interface IdentityProviderProps {
	children: ReactNode;
}

/**
 * Identity provider component that manages authentication state.
 * Automatically fetches dev tokens if no identity is set and handles token refresh.
 */
export function IdentityProvider({ children }: IdentityProviderProps) {
	const [tokens, setTokens] = useState<TokenState | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const refreshTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const isRefreshingRef = useRef(false);

	/**
	 * Configure the fetch client to include the access token in requests
	 */
	const configureApiClient = useCallback((accessToken: string | null) => {
		if (accessToken) {
			fetchClient.use({
				onRequest: ({ request }) => {
					request.headers.set("Authorization", `Bearer ${accessToken}`);
					return request;
				},
			});
		}
	}, []);

	/**
	 * Exchange dev STS token for access and refresh tokens
	 */
	const exchangeDevToken = useCallback(async (): Promise<TokenState | null> => {
		try {
			const response = await fetchClient.POST(
				"/api/identity/v1/sts/{sts_config_id}",
				{
					params: {
						path: {
							sts_config_id: "dev",
						},
					},
				},
			);

			if (response.error) {
				throw new Error(
					response.error.message || "Failed to exchange dev token",
				);
			}

			if (!response.data) {
				throw new Error("No data returned from STS exchange");
			}

			const { access_token, refresh_token, expires_in } = response.data;
			const expiresAt = Date.now() + expires_in * 1000;

			return {
				accessToken: access_token,
				refreshToken: refresh_token,
				expiresAt,
			};
		} catch (err) {
			const message =
				err instanceof Error
					? err.message
					: "Unknown error during token exchange";
			console.error("Failed to exchange dev token:", message);
			throw err;
		}
	}, []);

	/**
	 * Refresh the access token using the refresh token
	 */
	const refreshAccessToken = useCallback(
		async (refreshToken: string): Promise<TokenState | null> => {
			try {
				const response = await fetchClient.POST(
					"/api/identity/v1/auth/refresh",
					{
						body: {
							refresh_token: refreshToken,
						},
					},
				);

				if (response.error) {
					throw new Error(response.error.message || "Failed to refresh token");
				}

				if (!response.data) {
					throw new Error("No data returned from token refresh");
				}

				const { access_token, refresh_token, expires_in } = response.data;
				const expiresAt = Date.now() + expires_in * 1000;

				return {
					accessToken: access_token,
					// Use new refresh token if provided, otherwise keep the old one
					refreshToken: refresh_token || refreshToken,
					expiresAt,
				};
			} catch (err) {
				const message =
					err instanceof Error
						? err.message
						: "Unknown error during token refresh";
				console.error("Failed to refresh token:", message);
				throw err;
			}
		},
		[],
	);

	/**
	 * Schedule a token refresh before expiry
	 */
	const scheduleTokenRefresh = useCallback((currentTokens: TokenState) => {
		// Clear any existing timeout
		if (refreshTimeoutRef.current) {
			clearTimeout(refreshTimeoutRef.current);
			refreshTimeoutRef.current = null;
		}

		const timeUntilRefresh =
			currentTokens.expiresAt - Date.now() - REFRESH_BUFFER_MS;

		if (timeUntilRefresh <= 0) {
			// Token is already expired or about to expire, refresh immediately
			performTokenRefresh(currentTokens);
			return;
		}

		refreshTimeoutRef.current = setTimeout(() => {
			performTokenRefresh(currentTokens);
		}, timeUntilRefresh);
	}, []);

	/**
	 * Perform the actual token refresh
	 */
	const performTokenRefresh = useCallback(
		async (currentTokens: TokenState) => {
			if (isRefreshingRef.current) {
				return;
			}

			isRefreshingRef.current = true;

			try {
				const newTokens = await refreshAccessToken(currentTokens.refreshToken);
				if (newTokens) {
					setTokens(newTokens);
					saveTokensToStorage(newTokens);
					configureApiClient(newTokens.accessToken);
					scheduleTokenRefresh(newTokens);
				}
			} catch (err) {
				// If refresh fails, try to get new dev tokens
				console.warn("Token refresh failed, attempting to get new dev tokens");
				try {
					const newTokens = await exchangeDevToken();
					if (newTokens) {
						setTokens(newTokens);
						saveTokensToStorage(newTokens);
						configureApiClient(newTokens.accessToken);
						scheduleTokenRefresh(newTokens);
					}
				} catch (devErr) {
					setError("Authentication failed. Please try again.");
					setTokens(null);
					clearTokensFromStorage();
				}
			} finally {
				isRefreshingRef.current = false;
			}
		},
		[
			refreshAccessToken,
			exchangeDevToken,
			configureApiClient,
			scheduleTokenRefresh,
		],
	);

	/**
	 * Logout and clear all tokens
	 */
	const logout = useCallback(() => {
		if (refreshTimeoutRef.current) {
			clearTimeout(refreshTimeoutRef.current);
			refreshTimeoutRef.current = null;
		}
		setTokens(null);
		clearTokensFromStorage();
		setError(null);
	}, []);

	/**
	 * Initialize authentication on mount
	 */
	useEffect(() => {
		const initAuth = async () => {
			setIsLoading(true);
			setError(null);

			try {
				// Check for existing tokens in storage
				const storedTokens = loadTokensFromStorage();

				if (storedTokens && !isTokenExpired(storedTokens)) {
					// Valid tokens exist, use them
					setTokens(storedTokens);
					configureApiClient(storedTokens.accessToken);
					scheduleTokenRefresh(storedTokens);
				} else if (storedTokens && storedTokens.refreshToken) {
					// Tokens expired but we have a refresh token, try to refresh
					try {
						const newTokens = await refreshAccessToken(
							storedTokens.refreshToken,
						);
						if (newTokens) {
							setTokens(newTokens);
							saveTokensToStorage(newTokens);
							configureApiClient(newTokens.accessToken);
							scheduleTokenRefresh(newTokens);
						}
					} catch {
						// Refresh failed, get new dev tokens
						const newTokens = await exchangeDevToken();
						if (newTokens) {
							setTokens(newTokens);
							saveTokensToStorage(newTokens);
							configureApiClient(newTokens.accessToken);
							scheduleTokenRefresh(newTokens);
						}
					}
				} else {
					// No tokens, get dev tokens
					const newTokens = await exchangeDevToken();
					if (newTokens) {
						setTokens(newTokens);
						saveTokensToStorage(newTokens);
						configureApiClient(newTokens.accessToken);
						scheduleTokenRefresh(newTokens);
					}
				}
			} catch (err) {
				const message =
					err instanceof Error
						? err.message
						: "Authentication initialization failed";
				setError(message);
				console.error("Auth initialization error:", err);
			} finally {
				setIsLoading(false);
			}
		};

		initAuth();

		// Cleanup on unmount
		return () => {
			if (refreshTimeoutRef.current) {
				clearTimeout(refreshTimeoutRef.current);
			}
		};
	}, [
		configureApiClient,
		exchangeDevToken,
		refreshAccessToken,
		scheduleTokenRefresh,
	]);

	const value: IdentityContextValue = {
		isAuthenticated: tokens !== null,
		isLoading,
		error,
		accessToken: tokens?.accessToken ?? null,
		logout,
	};

	return (
		<IdentityContext.Provider value={value}>
			{children}
		</IdentityContext.Provider>
	);
}
