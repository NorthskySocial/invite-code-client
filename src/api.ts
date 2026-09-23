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

export const mockApiService = {
  login: async (username: string, _password: string): Promise<{ data: LoginResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    if (username === 'new-user') {
      return {
        data: {
          username: 'new-user',
          otp_enabled: false,
          otp_verified: false,
          otp_auth_url: 'otpauth://totp/InviteCode:new-user?secret=MOCKSECRET&issuer=InviteCode',
        },
      };
    }
    return {
      data: {
        username: 'demo-user',
        otp_enabled: true,
        otp_verified: true,
        otp_auth_url: null,
      },
    };
  },

  getInviteCodes: async (): Promise<{ data: InviteCodes }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    const codes: InviteCode[] = [
      {
        code: 'DEMO-123',
        available: 1,
        disabled: false,
        forAccount: 'demo-user',
        createdBy: 'demo-user',
        createdAt: new Date().toISOString(),
        uses: [],
      },
      {
        code: 'USED-456',
        available: 0,
        disabled: false,
        forAccount: 'demo-user',
        createdBy: 'admin',
        createdAt: new Date(Date.now() - 86400000).toISOString(),
        uses: [
          {
            usedBy: 'did:plc:mockuser',
            usedByHandle: 'mockuser.bsky.social',
            usedByEmail: null,
            usedAt: new Date().toISOString(),
            usedByDeactivated: false,
          },
        ],
      },
      {
        code: 'GONE-321',
        available: 0,
        disabled: false,
        forAccount: 'demo-user',
        createdBy: 'admin',
        createdAt: new Date(Date.now() - 259200000).toISOString(),
        uses: [
          {
            usedBy: 'did:plc:mockgone',
            usedByHandle: 'mockgone.bsky.social',
            usedByEmail: null,
            usedAt: new Date(Date.now() - 172800000).toISOString(),
            usedByDeactivated: true,
          },
        ],
      },
      {
        code: 'DISABLED-789',
        available: 1,
        disabled: true,
        forAccount: 'demo-user',
        createdBy: 'admin',
        createdAt: new Date(Date.now() - 172800000).toISOString(),
        uses: [],
      },
    ];
    return { data: { codes, cursor: null } };
  },

  createInviteCodes: async (count: number): Promise<{ data: void }> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    console.log(`Mock: Created ${count} invite codes`);
    return { data: undefined };
  },

  disableInviteCode: async (code: string): Promise<{ data: void }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    console.log(`Mock: Disabled code ${code}`);
    return { data: undefined };
  },

  validateOtp: async (_token: string): Promise<{ data: ValidateOtpResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return { data: { otp_valid: true } };
  },

  verifyOtp: async (_token: string): Promise<{ data: VerifyOtpResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return {
      data: {
        otp_verified: true,
        user: {
          username: 'new-user',
          otp_enabled: true,
          otp_verified: true,
          otp_auth_url: null,
        },
      },
    };
  },

  getAdmins: async (): Promise<{ data: AdminsResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    const admins: Admin[] = JSON.parse(
      localStorage.getItem('mock_admins') ||
        JSON.stringify([
          { username: 'admin', otp_enabled: true, otp_verified: true },
          { username: 'demo-user', otp_enabled: true, otp_verified: true },
        ])
    );
    return { data: { status: 'success', admins } };
  },

  addAdmin: async (username: string): Promise<{ data: AddAdminResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    const admins = JSON.parse(
      localStorage.getItem('mock_admins') ||
        JSON.stringify([
          { username: 'admin', otp_enabled: true, otp_verified: true },
          { username: 'demo-user', otp_enabled: true, otp_verified: true },
        ])
    );
    if (!admins.find((a: Admin) => a.username === username)) {
      admins.push({ username, otp_enabled: false, otp_verified: false });
      localStorage.setItem('mock_admins', JSON.stringify(admins));
    }
    return {
      data: {
        status: 'success',
        message: 'Admin user created successfully',
        password: 'mock-generated-password-' + Math.random().toString(36).substring(7),
      },
    };
  },

  removeAdmin: async (username: string): Promise<{ data: RemoveAdminResponse }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    const admins = JSON.parse(
      localStorage.getItem('mock_admins') ||
        JSON.stringify([
          { username: 'admin', otp_enabled: true, otp_verified: true },
          { username: 'demo-user', otp_enabled: true, otp_verified: true },
        ])
    );
    const filtered = admins.filter((a: Admin) => a.username !== username);
    localStorage.setItem('mock_admins', JSON.stringify(filtered));
    return { data: { status: 'success', message: 'Admin user removed successfully' } };
  },
};
