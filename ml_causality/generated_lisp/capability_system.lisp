list
(defun has-capability? (list capability-name)
 (map-has-key? capability-name
  (get-field current-user-resource capabilities)))
(if (eq (get-context-value requested_action) read) true false)
(defun capability-grant (list actor resource permission)
 (list actor resource permission granted))
(defun capability-revoke (list actor resource permission)
 (list actor resource permission revoked))
(defun register-capability-type (list type_name schema)
 (list type_name schema registered))
