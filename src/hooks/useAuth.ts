import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  startLogin,
  logout as apiLogout,
  getAuthStatus,
} from "../api/auth";
import { useAuthStore } from "../store/auth.store";
import type { AuthStatus } from "../types/ipc";

const LOGGED_OUT: AuthStatus = {
  logged_in:    false,
  user_id:      null,
  display_name: null,
  email:        null,
  product:      null,
  image_url:    null,
};

export function useAuth() {
  const store = useAuthStore();
  const qc    = useQueryClient();

  const { data: status, isLoading } = useQuery({
    queryKey:           ["auth-status"],
    queryFn:            getAuthStatus,
    staleTime:          5 * 60 * 1_000,
    refetchOnWindowFocus: false,
  });

  useEffect(() => {
    if (status !== undefined) store.setFromStatus(status);
  }, [status]); // eslint-disable-line react-hooks/exhaustive-deps

  const { mutate: login, isPending: loggingIn } = useMutation({
    mutationFn: startLogin,
    onSuccess: (data) => {
      store.setFromStatus(data);
      qc.setQueryData(["auth-status"], data);
    },
  });

  const { mutate: logout, isPending: loggingOut } = useMutation({
    mutationFn: apiLogout,
    onSuccess: () => {
      store.clear();
      qc.setQueryData(["auth-status"], LOGGED_OUT);
    },
  });

  return {
    ...store,
    isLoading,
    login,
    logout,
    loggingIn,
    loggingOut,
  };
}
