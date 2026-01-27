import axios from 'axios';

const getBaseURL = () => {
  return localStorage.getItem('api_host') || import.meta.env.VITE_API_HOST || 'https://frontend.myapp.local/';
};

export const api = axios.create({
  baseURL: getBaseURL(),
  withCredentials: true,
});

export const updateApiBaseURL = (newBaseURL: string) => {
  api.defaults.baseURL = newBaseURL;
};

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

export interface Admin {
  username: string;
  createdAt: string;
}

export interface AdminsResponse {
  admins: Admin[];
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

export interface AddAdminResponse {
  status: string;
  message: string;
  password?: string;
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

  getAdmins: () =>
    api.get<AdminsResponse>('/api/admins'),

  addAdmin: (username: string) =>
    api.post<AddAdminResponse>('/api/admins', {username}),

  removeAdmin: (username: string) =>
    api.delete(`/api/admins/${username}`),

  resolveDid: (did: string) =>
    axios.get(`https://plc.directory/${did}`),
};

export const mockApiService = {
  login: async (_username: string, _password: string): Promise<{ data: LoginResponse }> => {
    await new Promise(resolve => setTimeout(resolve, 500));
    return {data: {token: 'mock-token', username: 'demo-user'}};
  },

  getInviteCodes: async (): Promise<{ data: InviteCodes }> => {
    await new Promise(resolve => setTimeout(resolve, 300));
    const codes: InviteCode[] = [
      {
        code: 'DEMO-123',
        available: 1,
        disabled: false,
        forAccount: 'demo-user',
        createdBy: 'demo-user',
        createdAt: new Date().toISOString(),
        uses: []
      },
      {
        code: 'USED-456',
        available: 0,
        disabled: false,
        forAccount: 'demo-user',
        createdBy: 'admin',
        createdAt: new Date(Date.now() - 86400000).toISOString(),
        uses: [{usedBy: 'did:plc:mockuser', usedAt: new Date().toISOString()}]
      },
      {
        code: 'DISABLED-789',
        available: 1,
        disabled: true,
        forAccount: 'demo-user',
        createdBy: 'admin',
        createdAt: new Date(Date.now() - 172800000).toISOString(),
        uses: []
      }
    ];
    return {data: {codes}};
  },

  createInviteCodes: async (count: number) => {
    await new Promise(resolve => setTimeout(resolve, 500));
    console.log(`Mock: Created ${count} invite codes`);
    return {data: {success: true}};
  },

  disableInviteCode: async (code: string) => {
    await new Promise(resolve => setTimeout(resolve, 300));
    console.log(`Mock: Disabled code ${code}`);
    return {data: {success: true}};
  },

  generateOtp: async (): Promise<{ data: GenerateOtpResponse }> => {
    await new Promise(resolve => setTimeout(resolve, 300));
    return {data: {qr_code: 'mock-qr-code'}};
  },

  validateOtp: async (_token: string) => {
    await new Promise(resolve => setTimeout(resolve, 300));
    return {data: {token: 'mock-token'}};
  },

  verifyOtp: async (_token: string) => {
    await new Promise(resolve => setTimeout(resolve, 300));
    return {data: {success: true}};
  },

  getAdmins: async (): Promise<{ data: AdminsResponse }> => {
    await new Promise(resolve => setTimeout(resolve, 300));
    const admins: Admin[] = JSON.parse(localStorage.getItem('mock_admins') || JSON.stringify([
      {username: 'admin', createdAt: new Date(Date.now() - 604800000).toISOString()},
      {username: 'demo-user', createdAt: new Date().toISOString()}
    ]));
    return {data: {admins}};
  },

  addAdmin: async (username: string): Promise<{ data: AddAdminResponse }> => {
    await new Promise(resolve => setTimeout(resolve, 500));
    const admins = JSON.parse(localStorage.getItem('mock_admins') || JSON.stringify([
      {username: 'admin', createdAt: new Date(Date.now() - 604800000).toISOString()},
      {username: 'demo-user', createdAt: new Date().toISOString()}
    ]));
    if (!admins.find((a: any) => a.username === username)) {
      admins.push({username, createdAt: new Date().toISOString()});
      localStorage.setItem('mock_admins', JSON.stringify(admins));
    }
    return {
      data: {
        status: 'success',
        message: 'Admin user created successfully',
        password: 'mock-generated-password-' + Math.random().toString(36).substring(7)
      }
    };
  },

  removeAdmin: async (username: string) => {
    await new Promise(resolve => setTimeout(resolve, 300));
    const admins = JSON.parse(localStorage.getItem('mock_admins') || JSON.stringify([
      {username: 'admin', createdAt: new Date(Date.now() - 604800000).toISOString()},
      {username: 'demo-user', createdAt: new Date().toISOString()}
    ]));
    const filtered = admins.filter((a: any) => a.username !== username);
    localStorage.setItem('mock_admins', JSON.stringify(filtered));
    return {data: {success: true}};
  },

  resolveDid: async (did: string) => {
    return {data: {handle: did.replace('did:plc:', '') + '.test'}};
  },
};
