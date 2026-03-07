import { redirect } from "@sveltejs/kit";
import { browser } from "$app/environment";
import { base } from "$app/paths";
import { API_BASE_URL } from "$lib/apis";

function getAuthToken() {
  if (!browser) return "";
  const match = document.cookie.match(
    new RegExp("(^| )_khun_ph_token=([^;]+)"),
  );
  return match ? match[2] : "";
}

export async function load({ fetch }) {
  const token = getAuthToken();
  const headers: Record<string, string> = { Accept: "application/json" };

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}/auth/me`, {
    headers,
    credentials: "include",
  });

  if (!response.ok) {
    throw redirect(307, `${base}/login`);
  }

  const data = await response.json();

  if (data.role !== "admin") {
    throw redirect(307, `${base}/dashboard`);
  }

  return {};
}
