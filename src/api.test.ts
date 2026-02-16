import { describe, it, expect, beforeAll, afterEach, afterAll } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { apiService } from './api';

const handlers = [
  http.post('https://frontend.myapp.local/api/auth/login', async ({ request }) => {
    const { username } = (await request.json()) as { username: string };
    if (username === 'testuser') {
      return HttpResponse.json({ token: 'fake-token', username: 'testuser' });
    }
    return new HttpResponse(null, { status: 401 });
  }),

  http.get('https://frontend.myapp.local/api/invite-codes', () => {
    return HttpResponse.json({
      codes: [
        {
          code: 'CODE1',
          available: 1,
          disabled: false,
          forAccount: 'admin',
          createdBy: 'admin',
          createdAt: '2026-01-25T08:02:05.614Z',
          uses: [],
        },
        {
          code: 'CODE2',
          available: 0,
          disabled: false,
          forAccount: 'admin',
          createdBy: 'admin',
          createdAt: '2026-01-25T08:02:05.614Z',
          uses: [{ usedBy: 'user1', usedAt: '2026-01-25T08:12:55.280Z' }],
        },
      ],
    });
  }),

  http.post('https://frontend.myapp.local/api/create-invite-codes', async ({ request }) => {
    const { codeCount } = (await request.json()) as { codeCount: number };
    return HttpResponse.json({ message: `Created ${codeCount} codes` });
  }),

  http.post('https://frontend.myapp.local/api/disable-invite-codes', async ({ request }) => {
    const { code } = (await request.json()) as { code: string };
    return HttpResponse.json({ message: `Disabled code ${code}` });
  }),

  http.get('https://frontend.myapp.local/api/auth/otp/generate', () => {
    return HttpResponse.json({ qr_code: 'fake-qr-code' });
  }),

  http.post('https://frontend.myapp.local/api/auth/otp/validate', () => {
    return HttpResponse.json({ success: true });
  }),

  http.post('https://frontend.myapp.local/api/auth/otp/verify', () => {
    return HttpResponse.json({ success: true });
  }),

  http.get('https://plc.directory/:did', ({ params }) => {
    return HttpResponse.json({
      id: params.did,
      alsoKnownAs: [`at://handle.test`],
    });
  }),
];

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('apiService', () => {
  it('login should return token and username', async () => {
    const response = await apiService.login('testuser', 'password');
    expect(response.data).toEqual({ token: 'fake-token', username: 'testuser' });
  });

  it('getInviteCodes should return list of invite codes', async () => {
    const response = await apiService.getInviteCodes();
    expect(response.data.codes).toHaveLength(2);
    expect(response.data.codes[0].code).toBe('CODE1');
  });

  it('createInviteCodes should send correct count', async () => {
    const response = await apiService.createInviteCodes(5);
    expect(response.data).toEqual({ message: 'Created 5 codes' });
  });

  it('disableInviteCode should send correct code', async () => {
    const response = await apiService.disableInviteCode('CODE1');
    expect(response.data).toEqual({ message: 'Disabled code CODE1' });
  });

  it('generateOtp should return qr_code', async () => {
    const response = await apiService.generateOtp();
    expect(response.data).toEqual({ qr_code: 'fake-qr-code' });
  });

  it('validateOtp should return success and include credentials', async () => {
    let credentialsIncluded = false;
    server.use(
      http.post('https://frontend.myapp.local/api/auth/otp/validate', ({ request }) => {
        credentialsIncluded = request.credentials === 'include';
        return HttpResponse.json({ success: true });
      })
    );
    const response = await apiService.validateOtp('123456');
    expect(response.data).toEqual({ success: true });
    expect(credentialsIncluded).toBe(true);
  });

  it('verifyOtp should return success', async () => {
    const response = await apiService.verifyOtp('123456');
    expect(response.data).toEqual({ success: true });
  });

  it('resolveDid should return PLC data', async () => {
    const response = await apiService.resolveDid('did:plc:123');
    expect(response.data).toEqual({
      id: 'did:plc:123',
      alsoKnownAs: ['at://handle.test'],
    });
  });
});
