import axios, { isAxiosError, AxiosResponse } from 'axios';

export { isAxiosError };

export const getBaseURL = () =>
  localStorage.getItem('api_host') ||
  import.meta.env.VITE_API_HOST ||
  'https://frontend.myapp.local/';

export const api = axios.create({
  baseURL: getBaseURL(),
  withCredentials: true,
});

export const updateApiBaseURL = (newBaseURL: string) => {
  api.defaults.baseURL = newBaseURL;
};
export interface InviteCodes {
  codes: InviteCode[];
  cursor?: string | null;
}

export interface InviteCode {
  code: string;
  available: number;
  disabled: boolean;
  forAccount: string;
  createdBy: string;
  createdAt: string;
  uses: {
    usedBy: string;
    usedByHandle: string | null;
    usedByEmail: string | null;
    usedAt: string;
    usedByDeactivated: boolean | null;
  }[];
}

export interface Admin {
  username: string;
  otp_enabled: boolean;
  otp_verified: boolean;
}

export interface AdminsResponse {
  status: string;
  admins: Admin[];
}

export interface LoginResponse {
  username: string;
  otp_enabled: boolean;
  otp_verified: boolean;
  otp_auth_url: string | null;
}

export interface AddAdminResponse {
  status: string;
  message: string;
  password: string;
}

export interface RemoveAdminResponse {
  status: string;
  message: string;
}

export interface ValidateOtpResponse {
  otp_valid: boolean;
}

export interface VerifyOtpResponse {
  otp_verified: boolean;
  user: LoginResponse;
}

export const apiService = {
  login: (username: string, password: string): Promise<AxiosResponse<LoginResponse>> =>
    api.post<LoginResponse>('/api/auth/login', { username, password }),

  logout: (): Promise<AxiosResponse<void>> => api.post<void>('/api/auth/logout'),

  getInviteCodes: (): Promise<AxiosResponse<InviteCodes>> =>
    api.get<InviteCodes>('/api/invite-codes'),

  createInviteCodes: (count: number): Promise<AxiosResponse<void>> =>
    api.post<void>('/api/create-invite-codes', { codeCount: count, useCount: 1 }),

  disableInviteCode: (code: string): Promise<AxiosResponse<void>> =>
    api.post<void>('/api/disable-invite-codes', { codes: [code], accounts: [] }),

  validateOtp: (token: string): Promise<AxiosResponse<ValidateOtpResponse>> =>
    api.post<ValidateOtpResponse>('/api/auth/otp/validate', { token }),

  verifyOtp: (token: string): Promise<AxiosResponse<VerifyOtpResponse>> =>
    api.post<VerifyOtpResponse>('/api/auth/otp/verify', { token }),

  getAdmins: (): Promise<AxiosResponse<AdminsResponse>> => api.get<AdminsResponse>('/api/admins'),

  addAdmin: (username: string): Promise<AxiosResponse<AddAdminResponse>> =>
    api.post<AddAdminResponse>('/api/admins', { username }),

  removeAdmin: (username: string): Promise<AxiosResponse<RemoveAdminResponse>> =>
    api.delete<RemoveAdminResponse>('/api/admins', { data: { username } }),
};
