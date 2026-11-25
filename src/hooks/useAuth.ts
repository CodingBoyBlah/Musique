import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useShallow } from "zustand/react/shallow";
import { getAuthStatus, getProfile } from "../api/auth";
import { appleLoginSession, currentAuthRequest, loginSession, logoutSession } from "../lib/authSession";
import { appleCancelSignIn } from "../api/apple";
import { errMsg } from "../lib/err";
import { toast } from "../store/toast.store";
import { useAuthStore } from "../store/auth.store";
import type { AuthStatus } from "../types/ipc";

function statusMatchesStore(
  status: AuthStatus,
  state: Pick<ReturnType<typeof useAuthStore.getState>, "loggedIn" | "userId" | "displayName" | "email" | "product" | "imageUrl" | "provider">,
) {
  return (
    status.logged_in === state.loggedIn &&
    status.user_id === state.userId &&
    status.display_name === state.displayName &&
    status.email === state.email &&
    status.product === state.product &&
    status.image_url === state.imageUrl &&
    (status.provider === undefined || status.provider === state.provider)
  );
}

export function useAuth() {
  const store = useAuthStore(
    useShallow((s) => ({
      loggedIn: s.loggedIn,
      userId: s.userId,
      displayName: s.displayName,
      email: s.email,
      product: s.product,
      imageUrl: s.imageUrl,
      provider: s.provider,
      phase: s.phase,
      generation: s.generation,
      loginLinks: s.loginLinks,
    })),
  );
  const qc = useQueryClient();
  const { isLoading } = useQuery({
    queryKey: ["auth-status"],
    queryFn: async () => {
      const generation = useAuthStore.getState().generation;
      const status = await getAuthStatus();
      const state = useAuthStore.getState();
      if (currentAuthRequest(generation) && !statusMatchesStore(status, state)) {
        state.setFromStatus(status);
      }
      return status;
    },
    enabled: store.phase === "idle",
    staleTime: 60_000,
    refetchOnWindowFocus: true,
    refetchInterval: 60_000,
  });

  // Retry incomplete login profiles and keep every account view in sync.
  useQuery({
    queryKey: ["account-profile", store.userId],
    queryFn: async () => {
      const generation = useAuthStore.getState().generation;
      const profile = await getProfile();
      const state = useAuthStore.getState();
      if (currentAuthRequest(generation) && state.loggedIn) {
        const status: AuthStatus = {
          logged_in: true, user_id: profile.id, display_name: profile.display_name,
          email: profile.email, product: profile.product, image_url: profile.image_url,
        };
        const productChanged = profile.product !== state.product;
        if (!statusMatchesStore(status, state)) {
          state.setFromStatus(status);
          qc.setQueryData(["auth-status"], status);
        }
        if (productChanged) {
          void qc.invalidateQueries({ queryKey: ["playback-backend"] });
        }
      }
      return profile;
    },
    enabled: store.loggedIn && store.phase === "idle",
    staleTime: 60_000,
    refetchOnWindowFocus: true,
    refetchInterval: store.product ? 300_000 : 15_000,
    retry: 2,
  });

  const { mutate: login } = useMutation({
    mutationFn: (openBrowser: boolean) => loginSession(qc, openBrowser),
    onError: (err) => { console.error("[auth] login failed:", err); toast.error(errMsg(err)); },
  });
  const { mutate: loginApple } = useMutation({
    mutationFn: () => appleLoginSession(qc),
    onError: (err) => {
      const message = errMsg(err);
      console.error("[auth] apple login failed:", err);
      // closing the window is a choice, not a failure
      if (!/cancelled/i.test(message)) toast.error(message);
    },
  });
  const { mutate: logout } = useMutation({
    mutationFn: () => logoutSession(qc),
    onError: (err) => {
      console.error("[auth] logout failed:", err);
      toast.error(`Could not finish signing out: ${errMsg(err)}`);
    },
  });
  return {
    ...store, isLoading,
    login: (openBrowser = true) => login(openBrowser), logout: () => logout(),
    loginApple: () => loginApple(),
    cancelAppleLogin: () => { void appleCancelSignIn(); },
    loggingIn: store.phase === "login", loggingOut: store.phase === "logout",
  };
}
