export const idlFactory = ({ IDL }) => {
  const MessagePayload = IDL.Record({
    'title' : IDL.Text,
    'body' : IDL.Text,
    'attachment_url' : IDL.Text,
  });
  const Message = IDL.Record({
    'id' : IDL.Nat64,
    'upvotes' : IDL.Nat64,
    'title' : IDL.Text,
    'updated_at' : IDL.Opt(IDL.Nat64),
    'downvoted_users' : IDL.Vec(IDL.Text),
    'body' : IDL.Text,
    'created_at' : IDL.Nat64,
    'upvoted_users' : IDL.Vec(IDL.Text),
    'downvotes' : IDL.Nat64,
    'attachment_url' : IDL.Text,
  });
  const Error = IDL.Variant({
    'AlreadyVoted' : IDL.Record({ 'msg' : IDL.Text }),
    'NotFound' : IDL.Record({ 'msg' : IDL.Text }),
    'UserNotFound' : IDL.Record({ 'msg' : IDL.Text }),
  });
  const Result = IDL.Variant({ 'Ok' : Message, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  return IDL.Service({
    'add_message' : IDL.Func([MessagePayload], [IDL.Opt(Message)], []),
    'delete_message' : IDL.Func([IDL.Nat64], [Result], []),
    'downvote_message' : IDL.Func([IDL.Nat64, IDL.Text], [Result_1], []),
    'get_message' : IDL.Func([IDL.Nat64], [Result], ['query']),
    'reward_upvote' : IDL.Func([IDL.Text], [Result_1], []),
    'update_message' : IDL.Func([IDL.Nat64, MessagePayload], [Result], []),
    'upvote_message' : IDL.Func([IDL.Nat64, IDL.Text], [Result_1], []),
  });
};
export const init = ({ IDL }) => { return []; };
