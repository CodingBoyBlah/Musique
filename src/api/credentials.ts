import { invoke } from "@tauri-apps/api/core";

export interface Credentials {
  client_id: string;
  has_secret: boolean;
}

export interface ValidationResult {
  valid: boolean;
  error: string | null;
}

export const saveCredentials = (
  clientId: string,
  clientSecret: string,
): Promise<void> => invoke("save_credentials", { clientId, clientSecret });

export const getCredentials = (): Promise<Credentials | null> =>
  invoke("get_credentials");

export const validateCredentials = (): Promise<ValidationResult> =>
  invoke("validate_credentials");

export const clearCredentials = (): Promise<void> =>
  invoke("clear_credentials");
