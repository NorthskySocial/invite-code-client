import {describe, it, expect, beforeAll, afterEach, afterAll, vi} from 'vitest';
import {render, screen, fireEvent, waitFor} from '@testing-library/react';
import {setupServer} from 'msw/node';
import {http, HttpResponse} from 'msw';
import App from './App';

const handlers = [
  http.post('https://frontend.myapp.local/api/auth/login', async ({request}) => {
    const {username, password} = await request.json() as any;
    if (username === 'admin' && password === 'password') {
      return HttpResponse.json({token: 'fake-token', username: 'admin'});
    }
    return new HttpResponse(JSON.stringify({error: 'Invalid credentials'}), {status: 401});
  }),
  http.get('https://frontend.myapp.local/api/invite-codes', () => {
    return HttpResponse.json({
      codes: [
        {
          code: 'ABC-123',
          available: 1,
          disabled: false,
          forAccount: 'admin',
          createdBy: 'admin',
          createdAt: new Date().toISOString(),
          uses: []
        }
      ]
    });
  }),
];

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  localStorage.clear();
});
afterAll(() => server.close());

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

describe('App Component', () => {
  it('renders login page by default', () => {
    render(<App/>);
    expect(screen.getByText(/Invites Client/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Username/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', {name: /Login/i})).toBeInTheDocument();
  });

  it('successfully logs in and shows dashboard', async () => {
    render(<App/>);

    fireEvent.change(screen.getByPlaceholderText(/Username/i), {target: {value: 'admin'}});
    fireEvent.change(screen.getByPlaceholderText(/Password/i), {target: {value: 'password'}});
    fireEvent.click(screen.getByRole('button', {name: /Login/i}));

    await waitFor(() => {
      expect(screen.getAllByText('ABC-123')[0]).toBeInTheDocument();
    });

    expect(screen.getAllByText('ABC-123')[0]).toBeInTheDocument();
  });

  it('shows error on failed login', async () => {
    render(<App/>);

    fireEvent.change(screen.getByPlaceholderText(/Username/i), {target: {value: 'wrong'}});
    fireEvent.change(screen.getByPlaceholderText(/Password/i), {target: {value: 'wrong'}});
    fireEvent.click(screen.getByRole('button', {name: /Login/i}));

    await waitFor(() => {
      expect(screen.getByText(/Invalid credentials/i)).toBeInTheDocument();
    });
  });

  it('handles invalid date values without crashing', async () => {
    server.use(
      http.get('https://frontend.myapp.local/api/invite-codes', () => {
        return HttpResponse.json({
          codes: [
            {
              code: 'INVALID-DATE',
              available: 1,
              disabled: false,
              forAccount: 'admin',
              createdBy: 'admin',
              createdAt: 'invalid',
              uses: []
            },
            {
              code: 'EMPTY-DATE',
              available: 1,
              disabled: false,
              forAccount: 'admin',
              createdBy: 'admin',
              createdAt: '',
              uses: []
            },
            {
              code: 'NULL-DATE',
              available: 0,
              disabled: false,
              forAccount: 'admin',
              createdBy: 'admin',
              createdAt: new Date().toISOString(),
              uses: [] // In real scenario, available: 0 would mean there's a use, but for test logic '-' check it's fine
            }
          ]
        });
      })
    );

    render(<App/>);

    fireEvent.change(screen.getByPlaceholderText(/Username/i), {target: {value: 'admin'}});
    fireEvent.change(screen.getByPlaceholderText(/Password/i), {target: {value: 'password'}});
    fireEvent.click(screen.getByRole('button', {name: /Login/i}));

    await waitFor(() => {
      expect(screen.getAllByText('INVALID-DATE')[0]).toBeInTheDocument();
      expect(screen.getAllByText('EMPTY-DATE')[0]).toBeInTheDocument();
      expect(screen.getAllByText('NULL-DATE')[0]).toBeInTheDocument();
    });

    expect(screen.getAllByText('Invalid Date')[0]).toBeInTheDocument();
    // 3 codes, each has 2 date columns.
    // In each version (desktop/mobile), we have:
    // 1. INVALID-DATE: created_at='invalid' -> 'Invalid Date', used_at=undefined -> '-'
    // 2. EMPTY-DATE: created_at='' -> '-', used_at=undefined -> '-'
    // 3. NULL-DATE: created_at=valid -> '...', used_at=undefined -> '-'
    // For USED-BY column:
    // 1. INVALID-DATE: usedBy=undefined -> '-'
    // 2. EMPTY-DATE: usedBy=undefined -> '-'
    // 3. NULL-DATE: usedBy=undefined -> '-'
    // So per code:
    // INVALID-DATE: created_at='Invalid Date', used_at='-', usedBy='-'
    // EMPTY-DATE: created_at='-', used_at='-', usedBy='-'
    // NULL-DATE: created_at='...', used_at='-', usedBy='-'
    // TOTAL '-' per code:
    // INVALID-DATE: 2 '-'
    // EMPTY-DATE: 3 '-'
    // NULL-DATE: 2 '-'
    // TOTAL per version: 7 '-'
    // TOTAL for both versions: 14 '-'
    expect(screen.getAllByText('-')).toHaveLength(14);
  });

  it('correctly identifies used codes based on the uses array', async () => {
    server.use(
      http.get('https://frontend.myapp.local/api/invite-codes', () => {
        return HttpResponse.json({
          codes: [
            {
              code: 'USED-VIA-ARRAY',
              available: 1, // Still 1 according to API, but has uses
              disabled: false,
              forAccount: 'admin',
              createdBy: 'admin',
              createdAt: new Date().toISOString(),
              uses: [{usedBy: 'did:plc:user1', usedAt: new Date().toISOString()}]
            }
          ]
        });
      })
    );

    render(<App/>);

    fireEvent.change(screen.getByPlaceholderText(/Username/i), {target: {value: 'admin'}});
    fireEvent.change(screen.getByPlaceholderText(/Password/i), {target: {value: 'password'}});
    fireEvent.click(screen.getByRole('button', {name: /Login/i}));

    await waitFor(() => {
      expect(screen.getAllByText('USED-VIA-ARRAY')[0]).toBeInTheDocument();
    });

    // Check if the status is 'Used' (might be multiple 'Used' badges, so we check at least one exists)
    expect(screen.getAllByText('Used')[0]).toBeInTheDocument();
  });
});
