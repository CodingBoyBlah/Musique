import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getAuthStatus, getProfile } from "../api/auth";
import { currentAuthRequest, loginSession, logoutSession } from "../lib/authSession";
import { errMsg } from "../lib/err";
import { toast } from "../store/toast.store";
import { useAuthStore } from "../store/auth.store";

export function useAuth() {
  const store = useAuthStore();
  const qc = useQueryClient();
  const { isLoading } = useQuery({
    queryKey: ["auth-status"],
    queryFn: async () => {
      const generation = useAuthStore.getState().generation;
      const status = await getAuthStatus();
      if (currentAuthRequest(generation)) useAuthStore.getState().setFromStatus(status);
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
      if (currentAuthRequest(generation) && useAuthStore.getState().loggedIn) {
        const status = {
          logged_in: true, user_id: profile.id, display_name: profile.display_name,
          email: profile.email, product: profile.product, image_url: profile.image_url,
        };
        useAuthStore.getState().setFromStatus(status);
        qc.setQueryData(["auth-status"], status);
        void qc.invalidateQueries({ queryKey: ["playback-backend"] });
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
    mutationFn: () => loginSession(qc),
    onError: (err) => { console.error("[auth] login failed:", err); toast.error(errMsg(err)); },
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
    login: () => login(), logout: () => logout(),
    loggingIn: store.phase === "login", loggingOut: store.phase === "logout",
  };
}
