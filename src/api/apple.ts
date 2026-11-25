import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "../types/ipc";

/** Opens Apple's sign-in page in a private window; resolves once the account is signed in. */
export const appleSignIn = (): Promise<AuthStatus> => invoke("apple_sign_in");

/** Signs in with a pasted media-user-token (or a whole Cookie header). */
export const appleSignInWithToken = (token: string): Promise<AuthStatus> =>
  invoke("apple_sign_in_with_token", { token });

/** Closes a sign-in window that is still open, which rejects the pending appleSignIn. */
export const appleCancelSignIn = (): Promise<void> => invoke("apple_cancel_sign_in");
