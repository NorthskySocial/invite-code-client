import axios from 'axios';

const api = axios.create({
  // baseURL: 'https://invites.northsky.social',
  baseURL: 'https://frontend.myapp.local/',
  // baseURL: 'http://localhost:9090',
  withCredentials: true,
});

// Interceptor to add token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export interface InviteCodes {
  cursor?: string;
  codes: InviteCode[];
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
    usedAt: string;
  }[];
}

export interface LoginResponse {
  token?: string;
  requires_2fa?: boolean;
  two_factor_token?: string;
  username?: string;
  otp_enabled?: boolean;
  otp_verified?: boolean;
  otp_base32?: string;
  otp_auth_url?: string;
}

export interface GenerateOtpResponse {
  qr_code: string;
}

export const apiService = {
  login: (username: string, password: string) =>
    api.post<LoginResponse>('/api/auth/login', {username, password}),

  getInviteCodes: () =>
    api.get<InviteCodes>('/api/invite-codes'),

  createInviteCodes: (count: number) =>
    api.post('/api/create-invite-codes', {codeCount: count}),

  disableInviteCode: (code: string) =>
    api.post('/api/disable-invite-codes', {code}),

  generateOtp: () =>
    api.get<GenerateOtpResponse>('/api/auth/otp/generate'),

  validateOtp: (token: string) =>
    api.post('/api/auth/otp/validate', {token}),

  verifyOtp: (token: string) =>
    api.post('/api/auth/otp/verify', {token}),

  resolveDid: (did: string) =>
    axios.get(`https://plc.directory/${did}`),
};
