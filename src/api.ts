import axios from 'axios';

const api = axios.create({
  // baseURL: 'https://invites.northsky.social',
  baseURL: 'http://localhost:9090',
});

// Interceptor to add token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export interface InviteCode {
  id: string;
  code: string;
  status: 'Used' | 'Unused' | 'Disabled';
  created_at: string;
  used_at?: string;
}

export interface LoginResponse {
  token: string;
}

export interface GenerateOtpResponse {
  qr_code: string;
}

export const apiService = {
  login: (username: string, password: string) =>
    api.post<LoginResponse>('/auth/login', {username, password}),

  getInviteCodes: () =>
    api.get<InviteCode[]>('/invite-codes'),

  createInviteCodes: (count: number) =>
    api.post('/create-invite-codes', {count}),

  disableInviteCode: (code: string) =>
    api.post('/disable-invite-codes', {code}),

  generateOtp: () =>
    api.get<GenerateOtpResponse>('/auth/otp/generate'),

  validateOtp: (token: string) =>
    api.post('/auth/otp/validate', {token}),

  verifyOtp: (token: string) =>
    api.post('/auth/otp/verify', {token}),
};
