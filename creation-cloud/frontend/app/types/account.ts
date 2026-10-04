export interface Session {
  session_id: string;
  account_id: string;
  email: string | null;
  admin_login_name: string | null;
  role: string;
  csrf_token: string;
}
export interface Profile {
  account_id: string;
  display_name: string;
  locale: string;
}
export interface CloudPage<T> {
  items: T[];
  total: number;
  page: number;
  size: number;
}
